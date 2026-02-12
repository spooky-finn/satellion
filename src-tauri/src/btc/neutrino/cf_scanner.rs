use std::collections::HashMap;

use bip157::IndexedBlock;

use crate::btc::{
    DerivedScript, address::DerivePath, neutrino::block_downloader::BlockDownloader, utxo::Utxo,
};

pub struct CompactFilterScanner {
    scripts_of_interest: HashMap<bip157::ScriptBuf, DerivePath>,
    block_downloader: BlockDownloader,
}

impl CompactFilterScanner {
    pub fn new(block_downloader: BlockDownloader) -> Self {
        Self {
            scripts_of_interest: HashMap::default(),
            block_downloader,
        }
    }

    pub fn block_downloader_mut(&mut self) -> &mut BlockDownloader {
        &mut self.block_downloader
    }

    pub fn add_script(&mut self, s: DerivedScript) {
        self.scripts_of_interest.insert(s.script, s.derive_path);
    }

    /// Handle a compact filter - check if it matches and queue block download
    pub async fn handle_filter(&self, filter: bip157::IndexedFilter) {
        assert!(
            self.scripts_of_interest.len() >= 1,
            "No scripts to check in the filter"
        );

        // Check if this filter matches any of our scripts
        if filter.contains_any(self.scripts_of_interest.keys()) {
            let block_hash = filter.block_hash();
            if let Err(e) = self.block_downloader.queue_block(block_hash).await {
                tracing::error!("Failed to queue block: {}", e);
            }
        }
    }

    pub fn extract_utxos_from_block(&self, indexed_block: &IndexedBlock) -> Vec<Utxo> {
        let block_height = indexed_block.height;
        let block_hash = indexed_block.block.block_hash();
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
        utxos
    }
}
