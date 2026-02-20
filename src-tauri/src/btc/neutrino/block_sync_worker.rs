// use std::{sync::Arc, time::Duration};

// use tokio::{
//     sync::{RwLock, Semaphore, mpsc},
//     time::{sleep, timeout},
// };

// use crate::btc::{
//     address::ScriptHolder,
//     neutrino::{EventEmitter, HeightUpdateStatus},
//     sync,
// };

// const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
// const MAX_RETRIES: u32 = 3;

// pub enum BlockRequestEvent {
//     Final(BlockHash, SyncUpdate),
//     Middle(BlockHash),
// }

// pub struct BlockRequestChannel {
//     pub tx: mpsc::UnboundedSender<BlockRequestEvent>,
//     pub rx: mpsc::UnboundedReceiver<BlockRequestEvent>,
// }

// impl Default for BlockRequestChannel {
//     fn default() -> Self {
//         let (tx, rx) = mpsc::unbounded_channel();
//         Self { tx, rx }
//     }
// }

// pub struct BlockSyncWorker {
//     requester: Requester,
//     concurrency: usize,
//     sync_tx: mpsc::UnboundedSender<sync::Event>,
//     script_holder: Arc<RwLock<ScriptHolder>>,
//     event_emitter: EventEmitter,
// }

// impl BlockSyncWorker {
//     pub fn new(
//         requester: Requester,
//         sync_tx: mpsc::UnboundedSender<sync::Event>,
//         script_holder: Arc<RwLock<ScriptHolder>>,
//         event_emitter: EventEmitter,
//     ) -> Self {
//         Self {
//             requester,
//             concurrency: 1,
//             sync_tx,
//             script_holder,
//             event_emitter,
//         }
//     }

//     pub async fn run(&self, mut block_req_rx: mpsc::UnboundedReceiver<BlockRequestEvent>) {
//         let semaphore = Arc::new(Semaphore::new(self.concurrency));
//         while let Some(request) = block_req_rx.recv().await {
//             let permit = semaphore.clone().acquire_owned().await.unwrap();
//             let requester = self.requester.clone();
//             let script_holder = self.script_holder.clone();
//             let sync_tx = self.sync_tx.clone();
//             let emitter = self.event_emitter.clone();

//             tokio::spawn(async move {
//                 let _permit = permit;
//                 match request {
//                     BlockRequestEvent::Final(block_hash, sync_update) => {
//                         Self::download_block(requester, script_holder, sync_tx.clone(), block_hash)
//                             .await;
//                         Self::complete(&emitter, sync_update, sync_tx, block_hash).await;
//                     }
//                     BlockRequestEvent::Middle(block_hash) => {
//                         Self::download_block(requester, script_holder, sync_tx, block_hash).await
//                     }
//                 }
//             });
//         }
//     }

//     async fn download_block(
//         requester: Requester,
//         script_holder: Arc<RwLock<ScriptHolder>>,
//         sync_tx: mpsc::UnboundedSender<sync::Event>,
//         block_hash: BlockHash,
//     ) {
//         if let Some(block) = download_with_retry(&requester, block_hash).await {
//             let utxos = script_holder.read().await.extract_utxos(&block);
//             if let Err(e) = sync_tx.send(sync::Event::NewUtxos(utxos)) {
//                 tracing::error!("Failed to send sync event: {}", e);
//             }
//         }
//     }

//     async fn complete(
//         emitter: &EventEmitter,
//         sync_update: SyncUpdate,
//         sync_tx: mpsc::UnboundedSender<sync::Event>,
//         block_hash: BlockHash,
//     ) {
//         let height = sync_update.tip.height;

//         if block_hash == sync_update.tip.hash {
//             let payload = sync::Result {
//                 update: sync_update,
//                 broadcast_min_fee_rate: FeeRate::from_sat_per_kwu(0),
//                 avg_fee_rate: FeeRate::from_sat_per_kwu(0),
//             };

//             if let Err(e) = sync_tx.send(sync::Event::ChainSynced(payload)) {
//                 tracing::error!("Failed to send FiltersSynced event: {}", e);
//                 return;
//             }

//             emitter.height_updated(height, HeightUpdateStatus::Completed);
//             emitter.cf_sync_progress(100.0);
//         }
//     }
// }

// async fn download_with_retry(requester: &Requester, hash: BlockHash) -> Option<IndexedBlock> {
//     for attempt in 1..=MAX_RETRIES {
//         match attempt_download(requester, hash).await {
//             Ok(block) => {
//                 tracing::debug!("Downloaded block {} (height={})", hash, block.height);
//                 return Some(block);
//             }
//             Err(e) => {
//                 tracing::warn!(
//                     "Download failed (attempt {}/{}): {}",
//                     attempt,
//                     MAX_RETRIES,
//                     e
//                 );
//                 if attempt < MAX_RETRIES {
//                     let backoff = Duration::from_secs(2 * attempt as u64);
//                     sleep(backoff).await;
//                 }
//             }
//         }
//     }

//     tracing::error!(
//         "Failed to download block {} after {} attempts",
//         hash,
//         MAX_RETRIES
//     );
//     None
// }

// async fn attempt_download(requester: &Requester, hash: BlockHash) -> Result<IndexedBlock, String> {
//     let fut = async {
//         let rx = requester
//             .request_block(hash)
//             .map_err(|e| format!("Request error: {}", e))?;

//         rx.await
//             .map_err(|_| "Response channel closed".to_string())?
//             .map_err(|e| format!("Fetch error: {}", e))
//     };

//     timeout(REQUEST_TIMEOUT, fut)
//         .await
//         .map_err(|_| "Request timeout".to_string())?
// }
