use std::sync::Arc;

use tokio::sync::mpsc;

use crate::{EventEmitterTrait, btc::wallet::sync, repository::ChainRepositoryTrait, session::SK};

#[derive(Debug)]
pub struct Channels {
    pub sync_event_tx: mpsc::UnboundedSender<sync::Event>,
    pub sync_event_rx: mpsc::UnboundedReceiver<sync::Event>,
}

impl Default for Channels {
    fn default() -> Self {
        let (sync_event_tx, sync_event_rx) = mpsc::unbounded_channel();
        Self {
            sync_event_tx,
            sync_event_rx,
        }
    }
}

pub struct SyncOrchestrator {
    sk: SK,
    chain_repository: Arc<dyn ChainRepositoryTrait>,
    event_emitter: Arc<dyn EventEmitterTrait>,
    channels: Channels,
}

impl SyncOrchestrator {
    pub fn new(
        sk: SK,
        chain_repository: Arc<dyn ChainRepositoryTrait>,
        event_emitter: Arc<dyn EventEmitterTrait>,
    ) -> Self {
        Self {
            channels: Channels::default(),
            sk,
            chain_repository,
            event_emitter,
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        while let Some(event) = self.channels.sync_event_rx.recv().await {
            if let Err(e) = self.handle_sync_event(event).await {
                tracing::error!("Error handling sync event: {}", e);
            }
        }
        tracing::info!("sync_event_rx closed, sync orchestrator stopping");
        Ok(())
    }

    pub fn transmitter(&self) -> mpsc::UnboundedSender<sync::Event> {
        self.channels.sync_event_tx.clone()
    }

    async fn handle_sync_event(&self, event: sync::Event) -> Result<(), String> {
        match event {
            sync::Event::ChainSynced(event) => {
                tracing::info!("Filters synced, height: {}", event.update.tip.height);

                let mut sk = self.sk.lock().await;
                sk.wallet()?.mutate_btc(|btc| {
                    btc.cfilter_scanner_height = event.update.tip.height;
                    btc.initial_sync_done = true;
                    btc.runtime.sync.result = Some(event);
                    Ok(())
                })?;
            }
            sync::Event::BlockHeader(header) => {
                tracing::debug!("New block {}", header.height);
                if let Err(e) = self.chain_repository.save_block_header(&header) {
                    tracing::error!("Failed to save block header: {}", e);
                }
            }
            sync::Event::NewUtxos(utxos) => {
                let mut sk = self.sk.lock().await;
                let wallet = sk.wallet()?;
                wallet.btc.insert_utxos(utxos.clone());

                utxos.iter().for_each(|each| {
                    self.event_emitter
                        .new_utxo(each.output.value.to_sat(), wallet.btc.total_balance());
                });
            }
        }
        Ok(())
    }
}
