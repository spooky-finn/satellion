use bip157::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub regtest: bool,
    pub min_peers: u8,
}

impl BitcoinConfig {
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
        Self {
            regtest: false,
            min_peers: 3,
        }
    }
}
