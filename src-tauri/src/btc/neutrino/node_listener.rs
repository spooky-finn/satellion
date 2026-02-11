use std::time::{Duration, Instant};
use tokio::{sync::mpsc, task};

use bip157::{Client, Event, FeeRate, IndexedFilter, chain::BlockHeaderChanges};

use super::{EventEmitter, HeightUpdateStatus};
use crate::{btc::sync, utils::Throttler};

pub struct ListenArgs {
    pub event_emitter: EventEmitter,
    pub client: Client,
    pub filters_tx: mpsc::UnboundedSender<IndexedFilter>,
    pub sync_event_tx: mpsc::UnboundedSender<sync::Event>,
}

/// Handle incoming neutrino events from node on separate thread
pub async fn listen_node(args: ListenArgs) {
    let ListenArgs {
        event_emitter,
        client,
        sync_event_tx,
        filters_tx,
    } = args;

    let Client {
        mut info_rx,
        mut warn_rx,
        mut event_rx,
        requester,
    } = client;

    let sync_start_time = Instant::now();
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
                        let broadcast_min_fee_rate = requester.broadcast_min_feerate().await.unwrap_or_else(|e| {
                            tracing::error!("Failed to get broadcast min feerate: {}", e);
                            FeeRate::from_sat_per_kwu(0)
                        });

                        let avg_fee_rate = {
                            let tip_hash = sync_update.tip().hash;
                            let requester_clone = requester.clone();
                            let handle = task::spawn(async move {
                                requester_clone.average_fee_rate(tip_hash).await
                            });

                            match handle.await {
                                Ok(Ok(rate)) => rate,
                                Ok(Err(e)) => {
                                    tracing::error!("Failed to get average fee rate: {}", e);
                                    FeeRate::from_sat_per_kwu(0)
                                }
                                Err(e) => {
                                    tracing::error!("Task panicked or was cancelled: {:?}", e);
                                    FeeRate::from_sat_per_kwu(0)
                                }
                            }
                        };

                        let sync_time = sync_start_time.elapsed().as_secs_f32();
                        tracing::info!("Total sync time: {sync_time} seconds");

                        let payload = sync::Result {
                            update: sync_update.clone(),
                            broadcast_min_fee_rate,
                            avg_fee_rate,
                        };
                        let event = sync::Event::FiltersSynced(payload);
                        sync_event_tx.send(event).unwrap_or_else(|e| {
                            tracing::error!("Failed to send sync event: {}", e);
                        });
                        event_emitter.height_updated(sync_update.tip.height, HeightUpdateStatus::Completed);
                    }
                    Event::ChainUpdate(changes) => {
                        let new_height = match changes {
                            BlockHeaderChanges::Connected(header) => {
                                sync_event_tx.send(sync::Event::BlockHeader(header)).unwrap_or_else(|e| {
                                    tracing::error!("Failed to send block header event: {}", e);
                                });

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
                        if let Err(e) = filters_tx.send(filter) {
                            tracing::error!("Failed to send filter to processor: {}", e);
                        }
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
