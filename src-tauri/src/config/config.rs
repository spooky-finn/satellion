use crate::{bitcoin::config::BitcoinConfig, ethereum::config::EthereumConfig};
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
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        let mut path = PathBuf::from(home);
        path.push(".satellion");
        path.push("config.json");
        Ok(path)
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
