use std::collections::HashMap;

use bip157::Requester;

use crate::{btc::utxo::UTxO, session::SK};

pub struct CompactFilterScanner {
    sk: SK,
    requester: Requester,
}

impl CompactFilterScanner {
    pub fn new(sk: SK, requester: Requester) -> Self {
        Self { sk, requester }
    }

    pub async fn handle(&self, filter: bip157::IndexedFilter) {
        let scripts_of_interes = {
            let mut sk = self.sk.lock().await;
            let wallet = match sk.take_session() {
                Ok(s) => &s.wallet,
                Err(_) => return,
            };
            wallet.btc.runtime.scripts_of_interes.clone()
        };

        let script_map: HashMap<_, _> = scripts_of_interes
            .into_iter()
            .map(|s| (s.script.clone(), s.derive_path))
            .collect();

        if !filter.contains_any(script_map.keys()) {
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
        let mut utxos: Vec<UTxO> = vec![];

        for tx in &indexed_block.block.txdata {
            for (vout, output) in tx.output.iter().enumerate() {
                if let Some(derive_path) = script_map.get(&output.script_pubkey) {
                    utxos.push(UTxO {
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

        let mut sk = self.sk.lock().await;
        let wallet = match sk.take_session() {
            Ok(s) => &mut s.wallet,
            Err(_) => return,
        };
        wallet.btc.insert_utxos(utxos);
    }

    pub async fn update_scanner_height(&self, height: u32) -> Result<(), String> {
        let mut sk = self.sk.lock().await;
        let wallet = match sk.take_session() {
            Ok(s) => &mut s.wallet,
            Err(e) => return Err(e),
        };
        wallet.btc.cfilter_scanner_height = height;
        wallet
            .persist()
            .map_err(|e| format!("Bitcoin sync: fail to save wallet: {}", e))
    }
}
