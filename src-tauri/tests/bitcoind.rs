use std::{fs, str::FromStr, thread, time::Duration};

use corepc_client::{bitcoin::Address, client_sync::Error};
use corepc_node::{Client, Conf, Node, client::bitcoin::Amount};

pub struct BitcoindHarness {
    pub node: Node,
}

impl BitcoindHarness {
    pub fn start() -> Result<Self, String> {
        Self::prepare();
        let exe = std::path::PathBuf::from(
            std::env::var("BITCOIND_EXE")
                .unwrap_or_else(|_| "/opt/homebrew/bin/bitcoind".to_string()),
        );

        let mut conf = Conf::default();

        conf.wallet = None;
        conf.attempts = 1;
        conf.args.extend([
            "-txindex",
            "-blockfilterindex=1",
            "-peerblockfilters",
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

    pub fn tips(
        &self,
    ) -> std::result::Result<corepc_node::vtype::GetChainTips, corepc_client::client_sync::Error>
    {
        self.client().get_chain_tips()
    }

    fn prepare() {
        if let Some(home_dir) = std::env::home_dir() {
            let nakamoto_path = home_dir.join(".nakamoto").join("regtest");

            if nakamoto_path.exists() {
                if let Err(e) = fs::remove_dir_all(&nakamoto_path) {
                    eprintln!("Failed to clean nakamoto dir at {:?}: {}", nakamoto_path, e);
                } else {
                    println!("Successfully cleaned: {:?}", nakamoto_path);
                }
            }
        } else {
            eprintln!("Could not resolve user home directory.");
        }
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
