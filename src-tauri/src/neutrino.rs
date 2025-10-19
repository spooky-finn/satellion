use crate::app_state::AppState;
use crate::db::BlockHeader;
use crate::repository::Repository;
use bip157::chain::{BlockHeaderChanges, ChainState, IndexedHeader};
use bip157::{BlockHash, Builder, Event, Header, TrustedPeer};
use bitcoin::blockdata::block::{TxMerkleNode, Version};
use bitcoin::pow::CompactTarget;
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
    pub fn connect_regtest(block_headers: Vec<BlockHeader>) -> Result<Self, String> {
        let socket_addr = match SocketAddrV4::from_str(REGTEST_PEER) {
            Ok(addr) => addr,
            Err(e) => {
                return Err(format!("Error parsing socket address: {e:?}"));
            }
        };
        let peer = TrustedPeer::from_socket_addr(socket_addr);
        let indexed_headers = block_headers
            .iter()
            .map(|h| IndexedHeader {
                height: h.height as u32,
                header: Header {
                    merkle_root: TxMerkleNode::from_str(&h.merkle_root).unwrap(),
                    prev_blockhash: BlockHash::from_str(&h.prev_blockhash).unwrap(),
                    time: h.time as u32,
                    version: Version::from_consensus(h.version as i32),
                    bits: CompactTarget::from_consensus(h.bits as u32),
                    nonce: h.nonce as u32,
                },
            })
            .collect();
        let chain_state = ChainState::Snapshot(indexed_headers);

        let (node, client) = Builder::new(bip157::Network::Regtest)
            .required_peers(1)
            .chain_state(chain_state)
            .add_peers(vec![peer])
            .response_timeout(Duration::from_secs(10))
            .build();
        Ok(Self { node, client })
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    mut client: bip157::Client,
    app_state: Arc<AppState>,
    repository: Repository,
) {
    let block_height = app_state.chain_height.clone();
    let sync_completed = app_state.sync_completed.clone();

    while let Some(event) = client.event_rx.recv().await {
        match event {
            Event::FiltersSynced(sync_update) => {
                *block_height.lock().unwrap() = sync_update.tip.height;
                *sync_completed.lock().unwrap() = true;
                println!("Synced to height: {}", sync_update.tip.height);
            }
            Event::ChainUpdate(changes) => {
                let new_height = match changes {
                    BlockHeaderChanges::Connected(header) => {
                        if let Err(e) = repository.save_block_header(header) {
                            eprintln!("Error inserting block: {e:?}");
                        }
                        Some(header.height)
                    }
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
            Event::Block(_block) => {}
            Event::IndexedFilter(_filter) => {}
        }
    }
}
