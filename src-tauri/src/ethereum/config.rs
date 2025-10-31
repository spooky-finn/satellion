use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
}

impl EthereumConfig {
    pub fn new() -> Self {
        Self {
            rpc_url: "https://ethereum-rpc.publicnode.com".to_string(),
        }
    }
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self::new()
    }
}
