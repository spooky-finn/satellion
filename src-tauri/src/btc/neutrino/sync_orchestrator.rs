use crate::{
    btc::{neutrino::EventEmitter, wallet::sync},
    repository::ChainRepository,
    session::{SK, Session},
};

#[derive(Clone)]
pub struct SyncOrchestrator {
    sk: SK,
    chain_repository: ChainRepository,
    event_emitter: EventEmitter,
}

impl SyncOrchestrator {
    pub fn new(sk: SK, chain_repository: ChainRepository, event_emitter: EventEmitter) -> Self {
        Self {
            sk,
            chain_repository,
            event_emitter,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let mut sync_event_rx = {
            let mut session_keeper = self.sk.lock().await;
            let session = session_keeper.take_session()?;
            session
                .wallet
                .btc
                .runtime
                .sync
                .channels
                .sync_event_rx
                .take()
                .ok_or("sync_event_rx has been moved")?
        };

        while let Some(event) = sync_event_rx.recv().await {
            if let Err(e) = self.handle_sync_event(event).await {
                tracing::error!("Error handling sync event: {}", e);
            }
        }

        tracing::info!("All channels closed, sync orchestrator stopping");
        Ok(())
    }

    async fn handle_sync_event(&self, event: sync::Event) -> Result<(), String> {
        match event {
            sync::Event::ChainSynced(event) => {
                tracing::info!("Filters synced, height: {}", event.update.tip.height);

                let mut session_keeper = self.sk.lock().await;
                let session = session_keeper.take_session()?;
                session.wallet.mutate_btc(|btc| {
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
                let mut session_keeper = self.sk.lock().await;
                let Session { wallet, .. } = session_keeper.take_session()?;

                utxos.iter().for_each(|each| {
                    self.event_emitter
                        .new_utxo(each.output.value.to_sat(), wallet.btc.total_balance());
                });
                wallet.btc.insert_utxos(utxos);
            }
        }
        Ok(())
    }
}
