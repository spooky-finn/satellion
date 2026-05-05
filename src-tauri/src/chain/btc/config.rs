use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BitcoinConfig {
    pub regtest: bool,
    pub electrum_server: Option<String>,
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
