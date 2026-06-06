use std::str::FromStr;

use bitcoin::{
    TxOut, Txid,
    hashes::{Hash, sha256},
};
use serde::Deserialize;
use serde_json::json;

use crate::chain::btc::{
    account::AddressPathMap, config::BitcoinConfig, providers::electrum_client::ElectrumClient,
    utxo::Utxo,
};

pub struct ElectrumAdapter {
    client: ElectrumClient,
}

impl ElectrumAdapter {
    pub fn new(config: BitcoinConfig) -> Self {
        Self {
            client: ElectrumClient::new(config),
        }
    }

    pub fn new_tor(proxy: &str) -> Self {
        Self {
            client: ElectrumClient::new_tor(proxy),
        }
    }

    pub async fn estimate_fee(&self, blocks: u32) -> Result<f64, String> {
        let raw = self
            .client
            .request("blockchain.estimatefee", vec![json!(blocks)])
            .await?;
        raw.as_f64()
            .map(|btc_per_kb| btc_per_kb * 100_000.0)
            .ok_or_else(|| "invalid fee response".to_string())
    }

    pub async fn get_utxos(&self, address_path_map: AddressPathMap) -> Result<Vec<Utxo>, String> {
        let addresses: Vec<_> = address_path_map.keys().collect();
        let calls: Vec<_> = addresses
            .iter()
            .map(|a| {
                (
                    "blockchain.scripthash.listunspent",
                    vec![json!(scripthash(a))],
                )
            })
            .collect();

        let results = self.client.batch(calls).await?;
        let mut all = Vec::new();

        for (address, raw) in addresses.iter().zip(results) {
            let utxos: Vec<RawUtxo> =
                serde_json::from_value(raw).map_err(|e| format!("parse utxos: {e}"))?;
            let derivation = address_path_map[*address].clone();
            for u in utxos {
                let txid = Txid::from_str(&u.tx_hash).map_err(|e| format!("txid: {e}"))?;
                all.push(Utxo {
                    tx_id: txid,
                    vout: u.tx_pos,
                    output: TxOut {
                        value: bitcoin::Amount::from_sat(u.value),
                        script_pubkey: address.script_pubkey(),
                    },
                    derivation: derivation.clone(),
                    height: u.height,
                });
            }
        }
        Ok(all)
    }

    pub async fn broadcast_tx(&self, tx: &bitcoin::Transaction) -> Result<String, String> {
        let hex = bitcoin::consensus::encode::serialize_hex(tx);
        let raw = self
            .client
            .request("blockchain.transaction.broadcast", vec![json!(hex)])
            .await?;
        raw.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "unexpected broadcast response".to_string())
    }
}

#[derive(Deserialize)]
struct RawUtxo {
    tx_hash: String,
    tx_pos: u32,
    value: u64,
    height: u32,
}

fn scripthash(address: &bitcoin::Address) -> String {
    let script = address.script_pubkey();
    let mut hash = sha256::Hash::hash(script.as_bytes()).to_byte_array();
    hash.reverse();
    hash.iter().map(|b| format!("{b:02x}")).collect()
}
