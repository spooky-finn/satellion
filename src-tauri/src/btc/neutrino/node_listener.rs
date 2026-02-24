use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

use bip157::{Client, Event, chain::BlockHeaderChanges};

use super::HeightUpdateStatus;
use crate::{
    btc::{
        neutrino::{EventEmitterTrait, block_sync_worker::BlockRequestEvent},
        sync,
    },
    utils::Throttler,
};

pub struct NeutrinoClientContext {
    pub event_emitter: Arc<dyn EventEmitterTrait>,
    pub sync_event_tx: mpsc::UnboundedSender<sync::Event>,
    pub block_req_tx: mpsc::UnboundedSender<BlockRequestEvent>,
}

struct NodeEventHandler {
    ctx: NeutrinoClientContext,
    sync_start_time: Instant,
    progress_throttler: Throttler,
    height_throttler: Throttler,
}

impl NodeEventHandler {
    fn new(ctx: NeutrinoClientContext) -> Self {
        Self {
            ctx,
            sync_start_time: Instant::now(),
            progress_throttler: Throttler::new(Duration::from_secs(1)),
            height_throttler: Throttler::new(Duration::from_secs(1)),
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
                if let Err(e) = self
                    .ctx
                    .sync_event_tx
                    .send(sync::Event::BlockFilter(filter))
                {
                    tracing::error!("Failed to send sync event: {}", e);
                }
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
            self.ctx
                .block_req_tx
                .send(BlockRequestEvent::Final(sync_update.tip.hash, sync_update))
                .expect("fail to add last block");
        }
    }

    fn handle_chain_update(&mut self, changes: BlockHeaderChanges) {
        let new_height = match changes {
            BlockHeaderChanges::Connected(header) => {
                if let Err(e) = self
                    .ctx
                    .sync_event_tx
                    .send(sync::Event::BlockHeader(header))
                {
                    tracing::error!("Failed to send sync event: {}", e);
                }
                Some(header.height)
            }
            BlockHeaderChanges::Reorganized { accepted, .. } => accepted.last().map(|h| h.height),
            BlockHeaderChanges::ForkAdded(_) => None,
        };

        if let Some(h) = new_height
            && self.height_throttler.should_emit()
        {
            self.ctx
                .event_emitter
                .height_updated(h, HeightUpdateStatus::Progress);
        }
    }

    fn handle_info(&mut self, info: bip157::Info) {
        match info {
            bip157::Info::Progress(progress) => {
                let pct = progress.percentage_complete();
                if pct != 0.0 && self.progress_throttler.should_emit() {
                    tracing::debug!("Block filter download progress: {:.1}%", pct);
                    self.ctx.event_emitter.cf_sync_progress(pct);
                }
            }
            bip157::Info::SuccessfulHandshake => {
                tracing::info!("Neutrino handshake with peer");
            }
            bip157::Info::ConnectionsMet => {
                tracing::info!("Neutrino is connected to all required peers.");
            }
            bip157::Info::BlockReceived(_) => {
                // Handled by block downloader
            }
        }
    }

    fn handle_warning(&self, warn: bip157::Warning) {
        tracing::warn!("Bitcoin sync: warning: {}", warn);
        self.ctx.event_emitter.node_warning(warn.to_string());
    }
}

pub async fn run_neutrino_client(client: Client, args: NeutrinoClientContext) {
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
