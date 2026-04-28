use std::{str::FromStr, thread, time::Duration};

use corepc_client::{bitcoin::Address, client_sync::Error};
use corepc_node::{Client, Conf, Node, client::bitcoin::Amount};
use satellion_lib::chain::btc::{self, key_derivation::KeyDerivationPath, utxo};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ScannedUtxo {
    pub amount: f64,
    // pub blockhash: String,
    // pub coinbase: bool,
    // pub confirmations: u64,
    // pub desc: String,
    pub height: u64,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: String,
    pub txid: String,
    pub vout: u64,
}

impl ScannedUtxo {
    pub fn to_domain(&self, derivation: KeyDerivationPath) -> btc::utxo::Utxo {
        let tx_id = bitcoin::Txid::from_str(&self.txid).expect("invalid txid");
        let vout = self.vout as usize;
        let script_pubkey =
            bitcoin::ScriptBuf::from_hex(&self.script_pub_key).expect("invalid script");
        let value = bitcoin::Amount::from_btc(self.amount).expect("invalid amount");
        let height = self.height as u32;
        btc::utxo::Utxo {
            tx_id,
            vout,
            output: bitcoin::TxOut {
                script_pubkey,
                value,
            },
            derivation,
            height,
        }
    }
}

/// Parse scantxoutset result into a Vec of ScannedUtxo
pub fn parse_scan_result(
    scan_result: serde_json::Value,
) -> Result<Vec<ScannedUtxo>, Box<dyn std::error::Error>> {
    let utxos = scan_result["unspents"]
        .as_array()
        .ok_or("expected unspents array")?
        .iter()
        .filter_map(|utxo| serde_json::from_value(utxo.clone()).ok())
        .collect();
    Ok(utxos)
}

pub struct BitcoindHarness {
    pub node: Node,
}

impl BitcoindHarness {
    pub fn start() -> Result<Self, String> {
        let exe = std::path::PathBuf::from(
            std::env::var("BITCOIND_EXE")
                .unwrap_or_else(|_| "/opt/homebrew/bin/bitcoind".to_string()),
        );

        let mut conf = Conf::default();
        conf.wallet = None;
        conf.attempts = 1;
        conf.view_stdout = false;

        conf.args.extend([
            "-txindex",
            "-connect=0", // CRITICAL: Don't connect to other nodes
            "-listen=1",  // Ensure we are the master
        ]);

        let node = Node::with_conf(exe, &conf).map_err(|e| e.to_string())?;

        // retry loop for RPC readiness
        let client = &node.client;
        const MAX_ATTEMPTS: u32 = 10;
        let mut attempt = 0;
        while attempt < MAX_ATTEMPTS {
            match client.call::<serde_json::Value>("getblockchaininfo", &[]) {
                Ok(info) => {
                    if info["blocks"] == 0 {
                        break;
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(500));
                }
            }
            attempt += 1;
            if attempt == MAX_ATTEMPTS {
                return Err(format!(
                    "Failed to connect to bitcoind RPC after {} attempts",
                    MAX_ATTEMPTS,
                ));
            }
        }

        // create wallet with descriptors=true
        let _ = client.call::<serde_json::Value>(
            "createwallet",
            &[
                serde_json::json!("default"),
                serde_json::json!(false),
                serde_json::json!(false),
                serde_json::json!(""),
                serde_json::json!(false),
                serde_json::json!(true),
            ],
        );

        // 3. DO NOT use existing block height. Verify it is 0.
        let info: serde_json::Value = node.client.call("getblockchaininfo", &[]).unwrap();
        if info["blocks"].as_u64().unwrap() != 0 {
            return Err(format!(
                "State leak! Node started at block {}",
                info["blocks"]
            ));
        }

        Ok(Self { node })
    }

    pub fn client(&self) -> &Client {
        &self.node.client
    }

    pub fn mine_blocks(&self, blocks: usize) -> Result<(), Error> {
        let addr = self.client().new_address()?;
        self.client().generate_to_address(blocks, &addr)?;
        Ok(())
    }

    pub fn fund_wallet(&self) -> Result<(), Error> {
        self.mine_blocks(101)
    }

    pub fn send_to(&self, address: &str, btc: f64) -> Result<(), Error> {
        self.client()
            .send_to_address(&parse_addr(address), Amount::from_btc(btc).unwrap())?;
        Ok(())
    }

    pub fn send_and_confirm(&self, address: &str, btc: f64) -> Result<(), Error> {
        self.send_to(address, btc)?;
        self.mine_blocks(1)?;
        Ok(())
    }

    pub fn balance(&self) -> Result<Amount, Error> {
        Ok(self.client().get_balance()?.balance().unwrap())
    }

    pub fn scanutxoset(
        &self,
        address: String,
        derivation: &KeyDerivationPath,
    ) -> Result<Vec<utxo::Utxo>, Error> {
        let scan_result = self.client().call::<serde_json::Value>(
            "scantxoutset",
            &[
                serde_json::json!("start"),
                serde_json::json!([format!("addr({})", address)]),
            ],
        )?;
        let utxos = parse_scan_result(scan_result).expect("failed to parse scan result");
        Ok(utxos
            .iter()
            .map(|each| each.to_domain(derivation.clone()))
            .collect())
    }
}

impl Drop for BitcoindHarness {
    fn drop(&mut self) {
        let _ = self.node.stop();
    }
}

fn parse_addr(addr: &str) -> Address {
    Address::from_str(addr)
        .expect("invalid address")
        .require_network(corepc_client::bitcoin::Network::Regtest)
        .expect("invalid network")
}
