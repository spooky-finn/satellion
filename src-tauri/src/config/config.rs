use crate::{btc::config::BitcoinConfig, eth::config::EthereumConfig};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ethereum: EthereumConfig,
    pub bitcoin: BitcoinConfig,
}

impl Config {
    pub fn new() -> Self {
        Self::load().unwrap_or_else(|_| Self::default())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path();
        if !config_path.exists() {
            let default_config = Self::create_config()?;
            return Ok(default_config);
        }
        let raw_config = fs::read_to_string(&config_path)?;
        let json_config: Config = serde_json::from_str(&raw_config)?;
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

    pub fn session_exp_duration() -> chrono::TimeDelta {
        chrono::TimeDelta::hours(1)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ethereum: EthereumConfig::default(),
            bitcoin: BitcoinConfig::default(),
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);
