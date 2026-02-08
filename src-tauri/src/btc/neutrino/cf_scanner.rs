use std::collections::HashMap;
use tokio::sync::mpsc;

use bip157::Requester;

use crate::btc::{DerivedScript, address::DerivePath, sync, utxo::Utxo};

pub struct CompactFilterScanner {
    requester: Requester,
    cfilter_rx: FiltersRx,
    scripts_rx: ScriptsRx,
    scripts_of_interest: HashMap<bip157::ScriptBuf, DerivePath>,
    sync_event_tx: SyncEventTx,
}

type ScriptsRx = mpsc::UnboundedReceiver<DerivedScript>;
type FiltersRx = mpsc::UnboundedReceiver<bip157::IndexedFilter>;
type SyncEventTx = mpsc::UnboundedSender<sync::Event>;

impl CompactFilterScanner {
    pub fn new(
        requester: Requester,
        scripts_rx: ScriptsRx,
        filter_rx: FiltersRx,
        sync_event_tx: SyncEventTx,
    ) -> Self {
        Self {
            requester,
            scripts_rx,
            cfilter_rx: filter_rx,
            sync_event_tx,
            scripts_of_interest: HashMap::default(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                // Receive new compact block filter
                Some(filter) = self.cfilter_rx.recv() => {
                    self.handle(filter).await;
                }

                // Receive new scripts
                Some(DerivedScript { script, derive_path } ) = self.scripts_rx.recv() => {
                    self.scripts_of_interest.insert(script, derive_path);
                }

                else => break,
            }
        }
    }

    async fn handle(&self, filter: bip157::IndexedFilter) {
        if self.scripts_of_interest.is_empty() {
            tracing::error!("Neutrino filter scanner: no scripts of interest");
            return;
        }

        // No interested transactions in this block, skip it
        if !filter.contains_any(self.scripts_of_interest.keys()) {
            return;
        }

        let block_hash = filter.block_hash();
        let indexed_block = match self.requester.get_block(block_hash).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Neutrino requester: get block: {}", e);
                return;
            }
        };
        let block_height = indexed_block.height;
        let mut utxos: Vec<Utxo> = vec![];

        for tx in &indexed_block.block.txdata {
            for (vout, output) in tx.output.iter().enumerate() {
                if let Some(derive_path) = self.scripts_of_interest.get(&output.script_pubkey) {
                    utxos.push(Utxo {
                        tx_id: tx.compute_wtxid(),
                        output: output.clone(),
                        vout,
                        derive_path: derive_path.clone(),
                        block: crate::btc::utxo::BlockHeader {
                            hash: block_hash,
                            height: block_height,
                        },
                    });
                }
            }
        }

        if !utxos.is_empty()
            && let Err(e) = self.sync_event_tx.send(sync::Event::NewUtxos(utxos))
        {
            tracing::error!("Fail to send NewUtxos event: {}", e);
        }
    }
}
