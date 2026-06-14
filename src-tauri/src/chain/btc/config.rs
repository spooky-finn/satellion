use bitcoin::Network;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type, JsonSchema)]
#[serde(default)]
#[schemars(title = "Bitcoin")]
pub struct BitcoinConfig {
    #[schemars(skip)]
    pub regtest: bool,
    /// Custom Electrum server URL. Leave blank to use the default.
    #[schemars(title = "Electrum Server")]
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
