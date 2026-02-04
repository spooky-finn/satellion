use std::{net::SocketAddrV4, str::FromStr};

use bip157::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub regtest: bool,
    regtest_peer_socket: String,

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

    pub fn regtest_peer_socket(&self) -> SocketAddrV4 {
        SocketAddrV4::from_str(&self.regtest_peer_socket)
            .expect("invalid config value regtest_peer_socket")
    }
}

impl Default for BitcoinConfig {
    fn default() -> Self {
        Self {
            regtest: false,
            min_peers: 3,
            regtest_peer_socket: "127.0.0.1:18444".to_string(),
        }
    }
}
