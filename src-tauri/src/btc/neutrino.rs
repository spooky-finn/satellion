use std::{collections::HashSet, net::SocketAddrV4, str::FromStr, sync::Arc, time::Duration};

use bip157::{
    BlockHash, Builder, Client, Event, Header, Network, TrustedPeer,
    chain::{BlockHeaderChanges, ChainState, IndexedHeader},
};
use bitcoin::{
    blockdata::block::{TxMerkleNode, Version},
    pow::CompactTarget,
};

use crate::{
    app_state::AppState,
    btc::{DerivedScript, utxo::UTxO},
    config::CONFIG,
    db::BlockHeader,
    repository::ChainRepository,
};

const REGTEST_PEER: &str = "127.0.0.1:18444";

#[derive(Clone)]
pub struct NeutrinoStarter {
    pub repository: ChainRepository,
}

impl NeutrinoStarter {
    pub fn new(repository: ChainRepository) -> Self {
        Self { repository }
    }

    pub async fn sync(&self, scripts_of_interes: HashSet<DerivedScript>) -> Result<(), String> {
        let block_headers = self
            .repository
            .get_block_headers(10)
            .map_err(|e| format!("Failed to load block headers: {}", e))?;

        let (network, trusted_peers) = NeutrinoStarter::select_network().await?;
        println!("starting neutrino for network {}", network);
        let neutrino = Neutrino::connect(network, trusted_peers, block_headers)
            .map_err(|e| format!("Failed to connect to regtest: {}", e))?;

        let node = neutrino.node;
        let client = neutrino.client;
        let app_state = Arc::new(AppState::new());
        let repository = Arc::new(self.repository.clone());

        tauri::async_runtime::spawn(async move {
            if let Err(e) = node.run().await {
                eprintln!("Neutrinos: {}", e);
            }
        });

        tauri::async_runtime::spawn(handle_chain_updates(
            client,
            app_state,
            repository,
            scripts_of_interes,
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
        let peers: Vec<TrustedPeer> = seeds.into_iter().map(TrustedPeer::from_ip).collect();
        Ok((bip157::Network::Bitcoin, peers))
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
                    version: Version::from_consensus(h.version),
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
    scripts_of_interes: HashSet<DerivedScript>,
) {
    let block_height = app_state.chain_height.clone();
    let sync_completed = app_state.sync_completed.clone();
    let Client { requester, .. } = client;

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
            Event::IndexedFilter(filter) => {
                let scripts_iter = scripts_of_interes.iter().map(|s| &s.script);
                if !filter.contains_any(scripts_iter) {
                    return;
                }

                let block_hash = filter.block_hash();
                let indexed_block = match requester.get_block(block_hash).await {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Error processing indexed filter: {}", e);
                        continue;
                    }
                };
                let block_height = indexed_block.height;

                let unspent_outputs = indexed_block
                    .block
                    .txdata
                    .iter()
                    .flat_map(|tx| {
                        scripts_of_interes.iter().flat_map(move |derived_script| {
                            tx.output
                                .iter()
                                .enumerate()
                                .filter(|(_, vout)| derived_script.script == vout.script_pubkey)
                                .map(move |(vout, output)| UTxO {
                                                                        tx_id: tx.compute_wtxid(),
                                    output: output.clone(),
                                    vout,
                                    derive_path: derived_script.derive_path.clone(),
block: crate::btc::utxo::BlockHeader {
                                        hash: block_hash,
                                        height: block_height,
                                    },
                                })
                        })
                    })
                    .collect::<Vec<UTxO>>();

                if let Err(e) = repository.insert_utxos(unspent_outputs).await {
                    eprintln!("Failed to insert UTXOs for block {}: {}", block_hash, e);
                }
            }
        }
    }
}
