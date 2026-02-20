use anyhow::Result;
use bitcoin::Address;
use corepc_node::{Client, Conf, Node, client::bitcoin::Amount};

pub struct BitcoindHarness {
    pub node: Node,
}

impl BitcoindHarness {
    pub fn start() -> Result<Self> {
        let exe = std::path::PathBuf::from(
            std::env::var("BITCOIND_EXE")
                .unwrap_or_else(|_| "/opt/homebrew/bin/bitcoind".to_string()),
        );

        let mut conf = Conf::default();
        conf.wallet = None;
        conf.args
            .extend(["-txindex", "-blockfilterindex=1", "-peerblockfilters"]);

        let node = Node::with_conf(exe, &conf)?;

        // createwallet with descriptors=true (required for Core 28+)
        let _ = node.client.call::<serde_json::Value>(
            "createwallet",
            &[
                serde_json::json!("default"),
                serde_json::json!(false),
                serde_json::json!(false),
                serde_json::json!(""),
                serde_json::json!(false),
                serde_json::json!(true), // descriptors
            ],
        );

        // Use raw Value — Core 28+ changed `warnings` from String to Array,
        // which breaks corepc-node 0.10.1's typed deserializer
        let info = node
            .client
            .call::<serde_json::Value>("getblockchaininfo", &[])?;
        assert_eq!(info["blocks"], 0);

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
}

impl Drop for BitcoindHarness {
    fn drop(&mut self) {
        let _ = self.node.stop();
    }
}
