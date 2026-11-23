use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

const WALLETCONNECT_PROJECT_ID: &str = "e31691ac09a633626153a0d9191362d8";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, JsonSchema)]
#[serde(default)]
#[schemars(title = "Ethereum")]
pub struct EthereumConfig {
    /// Ethereum JSON-RPC endpoint URL
    #[schemars(title = "RPC URL")]
    pub rpc_url: String,
    #[schemars(skip)]
    pub anvil: bool,
    /// Leave empty to use Satellion's built-in WalletConnect project.
    #[schemars(title = "WalletConnect Project ID")]
    pub walletconnect_project_id: String,
    /// Connect to Sepolia and Base Sepolia for WalletConnect testing.
    #[schemars(title = "WalletConnect Testnets")]
    pub walletconnect_testnets: bool,
}

impl EthereumConfig {
    pub fn walletconnect_project_id(&self) -> &str {
        let project_id = self.walletconnect_project_id.trim();
        if project_id.is_empty() {
            WALLETCONNECT_PROJECT_ID
        } else {
            project_id
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
        Self {
            rpc_url: "https://ethereum-rpc.publicnode.com".to_string(),
            anvil: false,
            walletconnect_project_id: String::new(),
            walletconnect_testnets: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walletconnect_defaults_for_missing_and_blank_values() {
        let mut config: EthereumConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.walletconnect_project_id(), WALLETCONNECT_PROJECT_ID);

        config.walletconnect_project_id = "  ".into();
        assert_eq!(config.walletconnect_project_id(), WALLETCONNECT_PROJECT_ID);

        config.walletconnect_project_id = " custom-project ".into();
        assert_eq!(config.walletconnect_project_id(), "custom-project");
    }
}
