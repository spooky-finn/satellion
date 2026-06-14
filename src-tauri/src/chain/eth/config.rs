use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema)]
#[serde(default)]
#[schemars(title = "Ethereum")]
pub struct EthereumConfig {
    /// Ethereum JSON-RPC endpoint URL
    #[schemars(title = "RPC URL")]
    pub rpc_url: String,
    #[schemars(skip)]
    pub anvil: bool,
}

impl EthereumConfig {
    pub fn anvil_bin(&self) -> PathBuf {
        let home = std::env::var("HOME").expect("env HOME is not set");
        let mut path = PathBuf::from(home);
        path.push(".foundry/bin/anvil");
        path
    }
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://ethereum-rpc.publicnode.com".to_string(),
            anvil: false,
        }
    }
}
