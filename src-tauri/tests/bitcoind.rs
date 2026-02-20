use anyhow::{Result, anyhow};
use bitcoin::Address;
use corepc_node::{Client, Conf, Node, client::bitcoin::Amount};
use std::{fs, thread, time::Duration};

pub struct BitcoindHarness {
    pub node: Node,
}

impl BitcoindHarness {
    pub fn start() -> Result<Self> {
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

        let node = Node::with_conf(exe, &conf)?;

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
                return Err(anyhow!(
                    "Failed to connect to bitcoind RPC after {} attempts",
                    MAX_ATTEMPTS
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
        let info: serde_json::Value = node.client.call("getblockchaininfo", &[])?;
        if info["blocks"].as_u64().unwrap() != 0 {
            return Err(anyhow!(
                "State leak! Node started at block {}",
                info["blocks"]
            ));
        }

        Ok(Self { node })
    }

    pub fn client(&self) -> &Client {
        &self.node.client
    }

    pub fn mine_blocks(&self, blocks: usize) -> Result<()> {
        let addr = self.client().new_address()?;
        self.client().generate_to_address(blocks, &addr)?;
        Ok(())
    }

    pub fn fund_wallet(&self) -> Result<()> {
        self.mine_blocks(101)
    }

    pub fn send_to(&self, address: &Address, btc: f64) -> Result<()> {
        self.client()
            .send_to_address(address, Amount::from_btc(btc)?)?;
        Ok(())
    }

    pub fn send_and_confirm(&self, address: &Address, btc: f64) -> Result<()> {
        self.send_to(address, btc)?;
        self.mine_blocks(1)?;
        Ok(())
    }

    pub fn new_address(&self) -> Result<Address> {
        Ok(self.client().new_address()?)
    }

    pub fn balance(&self) -> Result<Amount> {
        Ok(self.client().get_balance()?.balance()?)
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
