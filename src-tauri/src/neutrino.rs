use bip157::{Builder, Event, TrustedPeer};
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

const REGTEST_PEER: &str = "127.0.0.1:18444";

pub struct Neutrino {
    pub node: bip157::Node,
    pub client: bip157::Client,
}

impl Neutrino {
    pub fn connect_regtest() -> Result<Self, String> {
        let socket_addr = match SocketAddrV4::from_str(REGTEST_PEER) {
            Ok(addr) => addr,
            Err(e) => {
                return Err(format!("Error parsing socket address: {e:?}"));
            }
        };
        let peer = TrustedPeer::from_socket_addr(socket_addr);
        let (node, client) = Builder::new(bip157::Network::Regtest)
            .required_peers(1)
            .add_peers(vec![peer])
            .build();
        Ok(Self { node, client })
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_events(mut client: bip157::Client, block_height: Arc<RwLock<Option<u32>>>) {
    while let Some(event) = client.event_rx.recv().await {
        match event {
            Event::FiltersSynced(sync_update) => {
                *block_height.write().await = Some(sync_update.tip.height);
                println!("Synced to height: {}", sync_update.tip.height);
            }
            Event::ChainUpdate(changes) => {
                use bip157::chain::BlockHeaderChanges;
                let new_height = match changes {
                    BlockHeaderChanges::Connected(header) => Some(header.height),
                    BlockHeaderChanges::Reorganized { accepted, .. } => {
                        accepted.last().map(|h| h.height)
                    }
                    BlockHeaderChanges::ForkAdded(_) => None,
                };
                if let Some(h) = new_height {
                    *block_height.write().await = Some(h);
                    println!("Height updated: {}", h);
                }
            }
            _ => {}
        }
    }
}
