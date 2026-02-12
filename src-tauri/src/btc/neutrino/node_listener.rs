use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, mpsc};

use bip157::{Client, Event, FeeRate, chain::BlockHeaderChanges};

use super::{EventEmitter, HeightUpdateStatus};
use crate::{
    btc::{neutrino::CompactFilterScanner, sync},
    utils::Throttler,
};

pub type CfScanner = Arc<RwLock<CompactFilterScanner>>;

pub struct NeutrinoClientArgs {
    pub event_emitter: EventEmitter,
    pub sync_event_tx: mpsc::UnboundedSender<sync::Event>,
    pub cf_scanner: CfScanner,
}

struct NodeEventHandler {
    event_emitter: EventEmitter,
    sync_event_tx: mpsc::UnboundedSender<sync::Event>,
    cf_scanner: CfScanner,
    sync_start_time: Instant,
    progress_throttler: Throttler,
    height_throttler: Throttler,
    block_download_started: Arc<AtomicBool>,
}

impl NodeEventHandler {
    fn new(listen_args: NeutrinoClientArgs) -> Self {
        let NeutrinoClientArgs {
            event_emitter,
            sync_event_tx,
            cf_scanner,
        } = listen_args;
        Self {
            event_emitter,
            sync_event_tx,
            cf_scanner,
            sync_start_time: Instant::now(),
            progress_throttler: Throttler::new(Duration::from_secs(1)),
            height_throttler: Throttler::new(Duration::from_secs(1)),
            block_download_started: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::FiltersSynced(sync_update) => {
                self.handle_filters_synced(sync_update).await;
            }
            Event::ChainUpdate(changes) => {
                self.handle_chain_update(changes);
            }
            Event::IndexedFilter(filter) => {
                self.cf_scanner.read().await.handle_filter(filter).await;
            }
            // Block events are handled by the block downloader
            Event::Block(_) => {}
        }
    }

    async fn handle_filters_synced(&mut self, sync_update: bip157::SyncUpdate) {
        let filters_sync_t = self.sync_start_time.elapsed().as_secs_f32();
        let h: u32 = sync_update.tip.height;

        tracing::info!(
            "Bitcoin sync: cfilter synced: height {}, sync_time: {:.2}s",
            h,
            filters_sync_t
        );

        {
            self.cf_scanner
                .write()
                .await
                .block_downloader_mut()
                .queue_block(sync_update.tip.hash)
                .await
                .expect("fail to add last block");
        }

        if self
            .block_download_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.start_block_downloading(sync_update).await;
        }
    }

    async fn start_block_downloading(&self, sync_update: bip157::SyncUpdate) {
        tracing::info!("Starting block download pipeline");
        let num_workers = 1; // adjust for mainnet or regtest
        // Channel for downloaded blocks
        let (result_tx, mut result_rx) = mpsc::unbounded_channel();

        // Spawn the block downloader
        let downloader_handle = {
            let mut scanner = self.cf_scanner.write().await;
            scanner.block_downloader_mut().spawn(num_workers, result_tx)
        };

        tracing::info!("Block downloader started");

        // Spawn block processor
        let cf_scanner = self.cf_scanner.clone();
        let sync_tx = self.sync_event_tx.clone();
        let stop_height = sync_update.clone().tip.height;
        let event_emiter_clone = self.event_emitter.clone();
        let sync_event_tx_clone = self.sync_event_tx.clone();
        let processor_handle = tokio::spawn(async move {
            tracing::info!("Block processor started");
            let mut blocks_processed = 0;

            while let Some(block) = result_rx.recv().await {
                let utxos = cf_scanner.read().await.extract_utxos_from_block(&block);
                let utxos_len = utxos.len();
                if let Err(e) = sync_tx.send(sync::Event::NewUtxos(utxos)) {
                    tracing::error!("Failed to send sync event: {}", e);
                }

                blocks_processed += 1;
                if blocks_processed % 10 == 0 {
                    tracing::info!("Processed {} blocks", blocks_processed);
                }

                tracing::info!("received block {}, utxos {}", block.height, utxos_len);
                if block.height == sync_update.tip.height {
                    break;
                }
            }

            tracing::info!(
                "Block processor shutting down (processed {} blocks)",
                blocks_processed
            );

            let payload = sync::Result {
                update: sync_update,
                broadcast_min_fee_rate: FeeRate::from_sat_per_kwu(0),
                avg_fee_rate: FeeRate::from_sat_per_kwu(0),
            };
            if let Err(e) = sync_event_tx_clone.send(sync::Event::ChainSynced(payload)) {
                tracing::error!("Failed to send FiltersSynced event: {}", e);
                return;
            }
            event_emiter_clone.height_updated(stop_height, HeightUpdateStatus::Completed);
            event_emiter_clone.cf_sync_progress(100.0);
        });

        // Monitor tasks
        tokio::spawn(async move {
            if let Err(e) = downloader_handle.await {
                tracing::error!("Block downloader panicked: {:?}", e);
            }
            if let Err(e) = processor_handle.await {
                tracing::error!("Block processor panicked: {:?}", e);
            }
            tracing::info!("Block download pipeline terminated");
        });

        // wait until sync completes
    }

    fn handle_chain_update(&mut self, changes: BlockHeaderChanges) {
        let new_height = match changes {
            BlockHeaderChanges::Connected(header) => {
                if let Err(e) = self.sync_event_tx.send(sync::Event::BlockHeader(header)) {
                    tracing::error!("Failed to send BlockHeader event: {}", e);
                }
                Some(header.height)
            }
            BlockHeaderChanges::Reorganized { accepted, .. } => accepted.last().map(|h| h.height),
            BlockHeaderChanges::ForkAdded(_) => None,
        };

        if let Some(h) = new_height {
            if self.height_throttler.should_emit() {
                self.event_emitter
                    .height_updated(h, HeightUpdateStatus::Progress);
            }
        }
    }

    fn handle_info(&mut self, info: bip157::Info) {
        match info {
            bip157::Info::Progress(progress) => {
                let pct = progress.percentage_complete();
                if pct != 0.0 && self.progress_throttler.should_emit() {
                    tracing::debug!("Block filter download progress: {:.1}%", pct);
                    self.event_emitter.cf_sync_progress(pct);
                }
            }
            bip157::Info::SuccessfulHandshake => {
                tracing::debug!("Successful handshake with peer");
            }
            bip157::Info::ConnectionsMet => {
                tracing::debug!("Connection requirements met");
            }
            bip157::Info::BlockReceived(_) => {
                // Handled by block downloader
            }
        }
    }

    fn handle_warning(&self, warn: bip157::Warning) {
        tracing::warn!("Bitcoin sync: warning: {}", warn);
        self.event_emitter.node_warning(warn.to_string());
    }
}

pub async fn run_neutrino_client(client: Client, args: NeutrinoClientArgs) {
    let mut handler = NodeEventHandler::new(args);
    let Client {
        mut info_rx,
        mut warn_rx,
        mut event_rx,
        ..
    } = client;
    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    handler.handle_event(event).await;
                }
            }
            info = info_rx.recv() => {
                if let Some(info) = info {
                    handler.handle_info(info);
                }
            }
            warn = warn_rx.recv() => {
                if let Some(warn) = warn {
                    handler.handle_warning(warn);
                }
            }
        }
    }
}
