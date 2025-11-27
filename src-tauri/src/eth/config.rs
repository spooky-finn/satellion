use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub anvil: bool,
}

impl EthereumConfig {
    pub fn new() -> Self {
        Self {
            rpc_url: "https://ethereum-rpc.publicnode.com".to_string(),
            anvil: false,
        }
    }

    pub fn anvil_bin(&self) -> PathBuf {
        let home = std::env::var("HOME").expect("env HOME is not set");
        let mut path = PathBuf::from(home);
        path.push(".foundry/bin/anvil");
        path
    }
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self::new()
    }
}
