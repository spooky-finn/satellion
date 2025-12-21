use std::{net::SocketAddrV4, str::FromStr, sync::Arc, time::Duration};

use bip157::{
    BlockHash, Builder, Event, Header, Network, TrustedPeer,
    chain::{BlockHeaderChanges, ChainState, IndexedHeader},
};
use bitcoin::{
    blockdata::block::{TxMerkleNode, Version},
    pow::CompactTarget,
};

use crate::{
    app_state::AppState, btc, config::CONFIG, db::BlockHeader, repository::ChainRepository,
};

const REGTEST_PEER: &str = "127.0.0.1:18444";

pub struct NeutrinoStarter;

impl NeutrinoStarter {
    pub async fn new(repository: ChainRepository) -> Result<(), String> {
        let block_headers = repository
            .get_block_headers(10)
            .map_err(|e| format!("Failed to load block headers: {}", e))?;

        let (network, trusted_peers) = NeutrinoStarter::select_network().await?;
        let neutrino = Neutrino::connect(network, trusted_peers, block_headers)
            .map_err(|e| format!("Failed to connect to regtest: {}", e))?;

        let node = neutrino.node;
        let client = neutrino.client;
        let app_state = Arc::new(AppState::new());
        let repository = Arc::new(repository.clone());

        tauri::async_runtime::spawn(async move {
            if let Err(e) = node.run().await {
                eprintln!("Neutrino node error: {}", e);
            }
        });

        tauri::async_runtime::spawn(btc::neutrino::handle_chain_updates(
            client, app_state, repository,
        ));

        Ok(())
    }

    async fn select_network() -> Result<(Network, Vec<TrustedPeer>), String> {
        if CONFIG.bitcoin.regtest {
            let socket_addr = SocketAddrV4::from_str(REGTEST_PEER)
                .map_err(|e| format!("error parsing regtest socket address: {e:?}"))?;
            let peer = TrustedPeer::from_socket_addr(socket_addr);
            return Ok((bip157::Network::Regtest, vec![peer]));
        }

        let seeds = bip157::lookup_host("seed.bitcoin.sipa.be").await;
        let peers: Vec<TrustedPeer> = seeds
            .into_iter()
            .map(|addr| TrustedPeer::from_ip(addr))
            .collect();
        return Ok((bip157::Network::Bitcoin, peers));
    }
}

pub struct Neutrino {
    pub node: bip157::Node,
    pub client: bip157::Client,
}

impl Neutrino {
    pub fn connect(
        network: Network,
        trusted_peers: Vec<TrustedPeer>,
        block_headers: Vec<BlockHeader>,
    ) -> Result<Self, String> {
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
        let (node, client) = Builder::new(network)
            .required_peers(1)
            .chain_state(chain_state)
            .add_peers(trusted_peers)
            .response_timeout(Duration::from_secs(10))
            .build();

        Ok(Self { node, client })
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    mut client: bip157::Client,
    app_state: Arc<AppState>,
    repository: Arc<ChainRepository>,
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
