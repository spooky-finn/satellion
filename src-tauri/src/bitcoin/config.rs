use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub regtest: bool,
}

impl BitcoinConfig {
    pub fn new() -> Self {
        Self { regtest: true }
    }
}

impl Default for BitcoinConfig {
    fn default() -> Self {
        Self::new()
    }
}
