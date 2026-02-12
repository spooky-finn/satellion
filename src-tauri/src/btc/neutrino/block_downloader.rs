use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Semaphore, mpsc},
    time::{sleep, timeout},
};

use bip157::{BlockHash, IndexedBlock, Requester};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;

pub struct BlockDownloader {
    requester: Requester,
    block_queue_tx: mpsc::UnboundedSender<BlockHash>,
    block_queue_rx: Option<mpsc::UnboundedReceiver<BlockHash>>,
}

impl BlockDownloader {
    pub fn new(requester: Requester) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            requester,
            block_queue_tx: tx,
            block_queue_rx: Some(rx),
        }
    }

    /// Enqueue a block for download
    pub async fn queue_block(&self, block_hash: BlockHash) -> Result<(), String> {
        self.block_queue_tx
            .send(block_hash)
            .map_err(|_| "Block queue closed".to_string())
    }

    pub fn spawn(
        &mut self,
        concurrency: usize,
        result_tx: mpsc::UnboundedSender<IndexedBlock>,
    ) -> tokio::task::JoinHandle<()> {
        let mut queue_rx = self
            .block_queue_rx
            .take()
            .expect("block_queue_rx was taken");

        let requester = self.requester.clone();
        let semaphore = Arc::new(Semaphore::new(concurrency));

        tokio::spawn(async move {
            tracing::info!(
                "Block downloader spawned with (concurrency={})",
                concurrency
            );

            while let Some(block_hash) = queue_rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let requester = requester.clone();
                let result_tx = result_tx.clone();

                tokio::spawn(async move {
                    let _permit = permit;

                    if let Some(block) = download_with_retry(&requester, block_hash).await {
                        if let Err(e) = result_tx.send(block) {
                            tracing::error!("Failed to send block: {}", e);
                        }
                    }
                });
            }

            tracing::info!("Block downloader shutting down");
        })
    }
}

async fn download_with_retry(requester: &Requester, hash: BlockHash) -> Option<IndexedBlock> {
    for attempt in 1..=MAX_RETRIES {
        match attempt_download(requester, hash).await {
            Ok(block) => {
                tracing::debug!("Downloaded block {} (height={})", hash, block.height);
                return Some(block);
            }
            Err(e) => {
                tracing::warn!(
                    "Download failed (attempt {}/{}): {}",
                    attempt,
                    MAX_RETRIES,
                    e
                );
                if attempt < MAX_RETRIES {
                    let backoff = Duration::from_secs(2 * attempt as u64);
                    sleep(backoff).await;
                }
            }
        }
    }

    tracing::error!(
        "Failed to download block {} after {} attempts",
        hash,
        MAX_RETRIES
    );
    None
}

async fn attempt_download(requester: &Requester, hash: BlockHash) -> Result<IndexedBlock, String> {
    let fut = async {
        let rx = requester
            .request_block(hash)
            .map_err(|e| format!("Request error: {}", e))?;

        rx.await
            .map_err(|_| "Response channel closed".to_string())?
            .map_err(|e| format!("Fetch error: {}", e))
    };

    timeout(REQUEST_TIMEOUT, fut)
        .await
        .map_err(|_| "Request timeout".to_string())?
}
