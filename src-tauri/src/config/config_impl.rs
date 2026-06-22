use std::{fs, path::PathBuf, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::chain::{btc::config::BitcoinConfig, eth::config::EthereumConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema)]
#[serde(default)]
#[schemars(title = "Network")]
pub struct TorConfig {
    /// Route connections through Tor for enhanced privacy
    #[schemars(title = "Tor Network")]
    pub enabled: bool,
    /// SOCKS5 proxy address. Tor must be running locally.
    /// Bitcoin routes Electrum connections through this proxy;
    /// Ethereum routes the configured RPC URL through this proxy.
    #[schemars(title = "SOCKS5 Proxy")]
    pub socks5_proxy: String,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            socks5_proxy: "socks5://127.0.0.1:9050".to_string(),
        }
    }
}

/// Security and access settings
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema)]
#[schemars(title = "Security")]
pub struct ConfigSecurity {
    /// Derive private keys without including the wallet passphrase
    #[schemars(title = "Omit Passphrase from Private Key")]
    pub omit_passphrase_on_private_key: bool,

    /// Lock the wallet after this many minutes of inactivity
    #[schemars(title = "Session Timeout (minutes)", range(min = 1, max = 1440))]
    pub session_inactivity_timeout_mins: u32,
}

impl Default for ConfigSecurity {
    fn default() -> Self {
        Self {
            omit_passphrase_on_private_key: false,
            session_inactivity_timeout_mins: 30,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type, JsonSchema)]
#[serde(default)]
pub struct Config {
    pub eth: EthereumConfig,
    pub btc: BitcoinConfig,
    pub tor: TorConfig,
    pub security: ConfigSecurity,
}

impl Config {
    pub fn new() -> Self {
        let config = Self::load().unwrap_or_else(|_| Self::default());
        Self::ensure_wallets_dir();
        config
    }

    pub fn session_inactivity_timeout(&self) -> Duration {
        Duration::from_mins(self.security.session_inactivity_timeout_mins as u64)
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path();

        if !config_path.exists() {
            let default_config = Self::create_config()?;
            tracing::warn!(
                "Config file not found at {}, creating default config",
                config_path.display()
            );
            return Ok(default_config);
        }

        let raw_config = fs::read_to_string(&config_path)?;
        let json_config: Config = serde_json::from_str(&raw_config).unwrap_or_else(|e| {
            tracing::error!("fail to deserialize config {}", e);
            Self::default()
        });
        tracing::debug!("config {:?}", json_config);
        Ok(json_config)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_file_path();
        let payload = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to stringify config: {e}"))?;
        fs::write(&config_path, payload)
            .map_err(|e| format!("failed to save config to {}: {e}", config_path.display()))?;
        Ok(())
    }

    fn create_config() -> Result<Config, String> {
        let config_path = Self::config_file_path();
        let default_config = Self::default();
        let payload = serde_json::to_string_pretty(&default_config)
            .map_err(|e| format!("failed to stringify default config: {e}"))?;

        fs::write(&config_path, payload).map_err(|e| {
            format!(
                "failed to save default config to {}: {e}",
                config_path.display()
            )
        })?;
        Ok(default_config)
    }

    pub fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").expect("env HOME is not set");
        let mut path = PathBuf::from(home);
        path.push(".satellion");
        path
    }

    fn config_file_path() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("config.json");
        path
    }

    pub fn db_path() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("blockchain.db");
        path
    }

    pub fn wallets_dir() -> PathBuf {
        let mut path = Self::config_dir();
        path.push("wallets");
        path
    }

    pub fn ensure_wallets_dir() {
        fs::create_dir_all(Self::wallets_dir()).expect("Failed to create wallets directory");
    }

    pub fn xprk_passphrase<'a>(&self, passphrase: &'a str) -> &'a str {
        if self.security.omit_passphrase_on_private_key {
            ""
        } else {
            passphrase
        }
    }
}
