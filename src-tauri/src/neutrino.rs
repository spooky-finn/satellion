use crate::app_state::AppState;
use bip157::chain::BlockHeaderChanges;
use bip157::{Builder, Event, TrustedPeer};
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

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
            .response_timeout(Duration::from_secs(10))
            .build();
        Ok(Self { node, client })
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    mut event_rx: bip157::UnboundedReceiver<Event>,
    app_state: Arc<AppState>,
) {
    let block_height = app_state.chain_height.clone();
    let sync_completed = app_state.sync_completed.clone();

    while let Some(event) = event_rx.recv().await {
        match event {
            Event::FiltersSynced(sync_update) => {
                *block_height.lock().unwrap() = sync_update.tip.height;
                *sync_completed.lock().unwrap() = true;
                println!("Synced to height: {}", sync_update.tip.height);
            }
            Event::ChainUpdate(changes) => {
                let new_height = match changes {
                    BlockHeaderChanges::Connected(header) => Some(header.height),
                    BlockHeaderChanges::Reorganized { accepted, .. } => {
                        accepted.last().map(|h| h.height)
                    }
                    BlockHeaderChanges::ForkAdded(_) => None,
                };

                match new_height {
                    Some(h) => {
                        *block_height.lock().unwrap() = h;
                        *sync_completed.lock().unwrap() = false;
                    }
                    None => {
                        eprintln!("Chain error: no new height");
                        *sync_completed.lock().unwrap() = true;
                    }
                }
            }
            Event::Block(block) => {
                // println!("Block event: {block:?}");
            }
            Event::IndexedFilter(filter) => {
                // println!("Indexed filter event: {filter:?}");
            }
        }
    }
}
