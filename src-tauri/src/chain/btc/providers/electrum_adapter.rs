use std::str::FromStr;

use bitcoin::{
    TxOut, Txid,
    block::Header,
    consensus::deserialize,
    hashes::{Hash, sha256},
    hex::FromHex,
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

    pub async fn network_status(&self) -> Result<(u32, String), String> {
        let raw = self
            .client
            .request("blockchain.headers.subscribe", vec![])
            .await?;
        let tip: RawHeaderSubscription =
            serde_json::from_value(raw).map_err(|e| format!("parse header subscription: {e}"))?;
        let header_bytes =
            Vec::<u8>::from_hex(&tip.hex).map_err(|e| format!("parse block header hex: {e}"))?;
        let header: Header =
            deserialize(&header_bytes).map_err(|e| format!("parse block header: {e}"))?;

        Ok((tip.height, header.block_hash().to_string()))
    }

    /// Batch query the activity flag for the given addresses. Uses
    /// `blockchain.scripthash.get_history` (much smaller payload than
    /// `listunspent` for empty addresses) and returns one boolean per input
    /// address, in the same order.
    pub async fn batch_has_activity(
        &self,
        addresses: &[bitcoin::Address],
    ) -> Result<Vec<bool>, String> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }
        let calls: Vec<_> = addresses
            .iter()
            .map(|a| {
                (
                    "blockchain.scripthash.get_history",
                    vec![json!(scripthash(a))],
                )
            })
            .collect();
        let results = self.client.batch(calls).await?;
        results
            .into_iter()
            .map(|r| {
                r.as_array()
                    .map(|a| !a.is_empty())
                    .ok_or_else(|| "get_history: expected array".to_string())
            })
            .collect()
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
        let expected = tx.compute_txid();
        let txid = expected.to_string();
        let result = self
            .client
            .request("blockchain.transaction.broadcast", vec![json!(hex)])
            .await;
        let error = match result {
            Ok(raw) if raw.as_str() == Some(txid.as_str()) => return Ok(txid),
            Ok(raw) => match raw.as_str().and_then(|value| value.parse::<Txid>().ok()) {
                Some(actual) => format!("broadcast returned mismatched transaction ID {actual}"),
                None => "broadcast response did not contain a transaction ID".to_string(),
            },
            Err(error) => broadcast_diagnostic(&error, &hex),
        };
        tracing::warn!(%txid, error = %error, "Broadcast response inconclusive; checking transaction on Electrum");
        match self
            .client
            .request("blockchain.transaction.get", vec![json!(txid)])
            .await
        {
            Ok(raw) => {
                let found = raw
                    .as_str()
                    .and_then(|hex| Vec::<u8>::from_hex(hex).ok())
                    .and_then(|bytes| deserialize::<bitcoin::Transaction>(&bytes).ok());
                if found
                    .as_ref()
                    .is_some_and(|found| found.compute_txid() == expected)
                {
                    tracing::info!(%txid, "Electrum confirms transaction is available");
                    return Ok(txid);
                }
                Err(format!(
                    "{error}; transaction lookup returned an invalid or mismatched transaction"
                ))
            }
            Err(lookup_error) => {
                let lookup_error = broadcast_diagnostic(&lookup_error, &hex);
                Err(format!(
                    "{error}; transaction lookup failed: {lookup_error}"
                ))
            }
        }
    }
}

// Error strings are supplied by the server and may echo the request.
fn broadcast_diagnostic(error: &str, signed_hex: &str) -> String {
    error
        .replace(signed_hex, "[signed transaction]")
        .replace(&signed_hex.to_ascii_uppercase(), "[signed transaction]")
        .chars()
        .filter(|c| !c.is_control())
        .take(300)
        .collect()
}

#[derive(Deserialize)]
struct RawUtxo {
    tx_hash: String,
    tx_pos: u32,
    value: u64,
    height: u32,
}

#[derive(Deserialize)]
struct RawHeaderSubscription {
    height: u32,
    hex: String,
}

fn scripthash(address: &bitcoin::Address) -> String {
    let script = address.script_pubkey();
    let mut hash = sha256::Hash::hash(script.as_bytes()).to_byte_array();
    hash.reverse();
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn inconclusive_broadcast_is_reconciled_by_transaction_lookup() {
        let tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![bitcoin::TxIn::default()],
            output: vec![],
        };
        let txid = tx.compute_txid().to_string();
        let hex = bitcoin::consensus::encode::serialize_hex(&tx);
        let (client, server) = tokio::io::duplex(8192);
        let adapter = ElectrumAdapter {
            client: ElectrumClient::from_test_stream(client),
        };
        let task = tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(server);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "blockchain.transaction.broadcast");
            writer
                .write_all(b"{\"id\":1,\"result\":[]}\n")
                .await
                .unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "blockchain.transaction.get");
            let response = format!("{}\n", serde_json::json!({"id": 2, "result": hex}));
            writer.write_all(response.as_bytes()).await.unwrap();
        });
        assert_eq!(adapter.broadcast_tx(&tx).await.unwrap(), txid);
        task.await.unwrap();
    }
    #[tokio::test]
    async fn lookup_of_wrong_transaction_does_not_confirm_broadcast() {
        let tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![bitcoin::TxIn::default()],
            output: vec![],
        };
        let mut other = tx.clone();
        other.lock_time = bitcoin::absolute::LockTime::from_consensus(1);
        let hex = bitcoin::consensus::encode::serialize_hex(&other);
        let (client, server) = tokio::io::duplex(8192);
        let adapter = ElectrumAdapter {
            client: ElectrumClient::from_test_stream(client),
        };
        let task = tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(server);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            writer
                .write_all(b"{\"id\":1,\"result\":[]}\n")
                .await
                .unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let response = format!("{}\n", serde_json::json!({"id": 2, "result": hex}));
            writer.write_all(response.as_bytes()).await.unwrap();
        });
        assert!(
            adapter
                .broadcast_tx(&tx)
                .await
                .unwrap_err()
                .contains("mismatched transaction")
        );
        task.await.unwrap();
    }
    /// Run with an isolated regtest Electrum server and a signed transaction.
    #[tokio::test]
    #[ignore = "requires SATELLION_TEST_ELECTRUM and SATELLION_TEST_TX_HEX"]
    async fn regtest_broadcast_and_retry_same_transaction() {
        let addr = std::env::var("SATELLION_TEST_ELECTRUM").unwrap();
        let hex = std::env::var("SATELLION_TEST_TX_HEX").unwrap();
        let tx: bitcoin::Transaction = deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap();
        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let adapter = ElectrumAdapter {
            client: ElectrumClient::from_test_stream(stream),
        };
        let expected = tx.compute_txid().to_string();
        assert_eq!(adapter.broadcast_tx(&tx).await.unwrap(), expected);
        assert_eq!(adapter.broadcast_tx(&tx).await.unwrap(), expected);
    }
}
