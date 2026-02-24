use std::sync::Arc;

use bip157::IndexedFilter;
use tokio::sync::{RwLock, mpsc};

use crate::{
    EventEmitterTrait,
    btc::{address::ScriptHolder, neutrino::block_sync_worker::BlockRequestEvent, wallet::sync},
    db,
    repository::ChainRepositoryTrait,
    session::SK,
};

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
    script_holder: Arc<RwLock<ScriptHolder>>,
    channels: Channels,
    block_req_tx: mpsc::UnboundedSender<BlockRequestEvent>,
    wallet_birth_date: Option<u64>,
}

impl SyncOrchestrator {
    pub fn new(
        sk: SK,
        chain_repository: Arc<dyn ChainRepositoryTrait>,
        event_emitter: Arc<dyn EventEmitterTrait>,
        script_holder: Arc<RwLock<ScriptHolder>>,
        block_req_tx: mpsc::UnboundedSender<BlockRequestEvent>,
        wallet_birth_date: Option<u64>,
    ) -> Self {
        Self {
            channels: Channels::default(),
            script_holder,
            sk,
            chain_repository,
            event_emitter,
            block_req_tx,
            wallet_birth_date,
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

    fn should_persist_block(&self, block_time: u32) -> bool {
        self.wallet_birth_date
            .map(|birth| block_time as u64 >= birth)
            .unwrap_or(true)
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
                if !self.should_persist_block(header.header.time) {
                    return Ok(());
                }

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
            sync::Event::BlockFilter(f) => self.handle_filter(f).await,
        }
        Ok(())
    }

    /// Handle a compact filter - check if it matches and queue block download
    async fn handle_filter(&self, filter: IndexedFilter) {
        let script_holder = self.script_holder.read().await;
        assert!(
            script_holder.len() >= 1,
            "No scripts to check in the filter"
        );
        let scipts = script_holder.scripts();

        // Check if this filter matches any of our scripts
        if filter.contains_any(scipts) {
            let block_hash = filter.block_hash();
            if let Err(e) = self
                .block_req_tx
                .send(BlockRequestEvent::Middle(block_hash))
            {
                tracing::error!("fail to send to block_req_tx: {}", e);
            }
        }

        self.persist_cfilter(filter);
    }

    /// Persist filter only if block time >= wallet birth date
    fn should_persist_filter(&self, header: db::BlockHeader) -> bool {
        self.wallet_birth_date
            .map(|birth| header.time as u64 >= birth)
            .unwrap_or(true)
    }

    fn persist_cfilter(&self, filter: IndexedFilter) {
        let repo = &self.chain_repository;
        let header = match repo.get_block_header(filter.height()) {
            Ok(Some(header)) => header,
            Ok(None) => {
                tracing::warn!("Missing block header at height {}", filter.height());
                return;
            }
            Err(e) => {
                panic!("Failed to load block header: {}", e);
            }
        };

        if !self.should_persist_filter(header) {
            return;
        }

        let filter_bytes = filter.clone().into_contents();
        let block_hash = filter.block_hash();
        if let Err(e) = repo.save_compact_filter(&block_hash.to_string(), &filter_bytes) {
            tracing::error!(
                "Failed to persist compact filter for block {}: {}",
                block_hash,
                e
            );
        }
    }
}
