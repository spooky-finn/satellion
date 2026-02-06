use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bip157::{Client, Event, chain::BlockHeaderChanges};

use super::{CompactFilterScanner, EventEmitter, HeightUpdateStatus};
use crate::{repository::ChainRepository, utils::Throttler};

/// Handle incoming neutrino events from node on separate thread
pub async fn listen_neutrino_node(
    event_emitter: EventEmitter,
    client: bip157::Client,
    repository: Arc<ChainRepository>,
    cfilter_processor: CompactFilterScanner,
) {
    let Client {
        mut info_rx,
        mut warn_rx,
        mut event_rx,
        requester,
    } = client;
    // let mut block_counter = 0;
    let now = Instant::now();
    let mut progress_throttler = Throttler::new(Duration::from_secs(1));
    let mut height_throttler = Throttler::new(Duration::from_secs(1));
    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    match event {
                    Event::FiltersSynced(sync_update) => {
                        tracing::debug!("Bitcoin sync: cfilter synced: height {}", sync_update.tip.height);
                        // Request information from the node
                        match requester.broadcast_min_feerate().await {
                            Ok(fee) => tracing::info!("Minimum transaction broadcast fee rate: {:#}", fee),
                            Err(e) => tracing::error!("Failed to get broadcast min feerate: {}", e),
                        }
                        let sync_time = now.elapsed().as_secs_f32();
                        tracing::info!("Total sync time: {sync_time} seconds");
                        match requester.average_fee_rate(sync_update.tip().hash).await {
                            Ok(avg_fee_rate) => tracing::info!("Last block average fee rate: {:#}", avg_fee_rate),
                            Err(e) => tracing::error!("Failed to get average fee rate: {}", e),
                        }

                        event_emitter.height_updated(sync_update.tip.height, HeightUpdateStatus::Completed);
                        if let Err(e) = cfilter_processor.update_scanner_height(sync_update.tip.height).await {
                            tracing::error!(e);
                        }

                        // TODO: send event to the bitcoin wallet that inital sync done
                    }
                    Event::ChainUpdate(changes) => {
                        let new_height = match changes {
                            BlockHeaderChanges::Connected(header) => {
                                if let Err(err) = repository.insert_block_header(&header) &&
                                    !err.to_string().contains("UNIQUE constraint failed") {
                                        tracing::error!("Bitcoin sync: warning: failed to insert block header: {}", err);
                                    }
                                Some(header.height)
                            }
                            BlockHeaderChanges::Reorganized { accepted, .. } => {
                                accepted.last().map(|h| h.height)
                            }
                            BlockHeaderChanges::ForkAdded(_) => None,
                        };
                        if let Some(height) = new_height && height_throttler.should_emit() {
                            event_emitter.height_updated(height, HeightUpdateStatus::Progress);
                        }
                    }
                    Event::Block(_block) => {
                        tracing::debug!("Bitcoin sync: block {}", _block.height);
                    }
                    Event::IndexedFilter(filter) => {
                        cfilter_processor.handle(filter).await;
                    }
                    }
                }
            }

            info = info_rx.recv() => {
                if let Some(info) = info {
                    match info {
                        bip157::Info::SuccessfulHandshake => {},
                        bip157::Info::ConnectionsMet => {},
                        bip157::Info::Progress(progress) => {
                                let pct = progress.percentage_complete();
                                if pct != 0.0  && progress_throttler.should_emit() {
                                    tracing::debug!("Block filter download progress {}", pct);
                                    event_emitter.cf_sync_progress(pct)
                                }
                        },
                        bip157::Info::BlockReceived(_) => {},
                        }
                }
            }

            warn = warn_rx.recv() => {
                if let Some(warn) = warn {
                    tracing::warn!("Bitcoin sync: warning: {}", warn);
                    event_emitter.node_warning(warn.to_string())
                }
            }
        }
    }
}
