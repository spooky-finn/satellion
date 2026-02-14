use std::{fs, path::PathBuf, time::Duration};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{btc::config::BitcoinConfig, eth::config::EthereumConfig};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub ethereum: EthereumConfig,
    pub bitcoin: BitcoinConfig,
    /// Require a passphrase when generating private keys
    pub omit_passphrase_on_private_key: bool,
}

impl Config {
    pub fn new() -> Self {
        let config = Self::load().unwrap_or_else(|_| Self::default());
        Self::ensure_wallets_dir();
        config
    }

    pub fn session_inactivity_timeout(&self) -> Duration {
        Duration::from_mins(10)
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
        Ok(json_config)
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
        if self.omit_passphrase_on_private_key {
            ""
        } else {
            passphrase
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);
