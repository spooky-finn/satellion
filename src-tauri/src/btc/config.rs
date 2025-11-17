use bip157::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub regtest: bool,
}

impl BitcoinConfig {
    pub fn new() -> Self {
        Self { regtest: true }
    }

    pub fn network(&self) -> Network {
        if self.regtest {
            Network::Regtest
        } else {
            Network::Bitcoin
        }
    }
}

impl Default for BitcoinConfig {
    fn default() -> Self {
        Self::new()
    }
}
