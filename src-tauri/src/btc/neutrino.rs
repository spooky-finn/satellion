use std::{net::SocketAddrV4, str::FromStr, sync::Arc, time::Duration};

use bip157::{
    BlockHash, Builder, Client, Event, Header, Network, Requester, TrustedPeer,
    chain::{BlockHeaderChanges, ChainState, IndexedHeader},
};
use bitcoin::{
    blockdata::block::{TxMerkleNode, Version},
    pow::CompactTarget,
};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::{
    btc::utxo::UTxO, config::CONFIG, db::BlockHeader, repository::ChainRepository,
    session::SessionKeeper,
};

const REGTEST_PEER: &str = "127.0.0.1:18444";

pub const EVENT_HEIGHT_UPDATE: &str = "btc_sync";
pub const EVENT_SYNC_PROGRESS: &str = "btc_sync_progress";
pub const EVENT_SYNC_WARNING: &str = "btc_sync_warning";

#[derive(Clone)]
pub struct NeutrinoStarter {
    repository: ChainRepository,
    session_keeper: Arc<Mutex<SessionKeeper>>,
}

impl NeutrinoStarter {
    pub fn new(repository: ChainRepository, session_keeper: Arc<Mutex<SessionKeeper>>) -> Self {
        Self {
            repository,
            session_keeper,
        }
    }

    pub async fn sync(&self, app: AppHandle, wallet_name: String) -> Result<(), String> {
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
        let repository = Arc::new(self.repository.clone());

        tauri::async_runtime::spawn(async move {
            if let Err(e) = node.run().await {
                eprintln!("Neutrinos: {}", e);
            }
        });

        let cfilter_processor = CFilterProcessor {
            session_keeper: self.session_keeper.clone(),
            wallet_name,
            requester: client.requester.clone(),
        };

        tauri::async_runtime::spawn(handle_chain_updates(
            app,
            client,
            repository,
            cfilter_processor,
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

#[derive(Debug, Clone, Serialize, Type)]
enum HeightUpdateStatus {
    #[serde(rename = "in progress")]
    Progress,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
pub struct SyncHeightUpdateEvent {
    status: HeightUpdateStatus,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
pub struct SyncProgressEvent {
    progress: f32,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
pub struct SyncWarningEvent {
    msg: String,
}

pub struct CFilterProcessor {
    session_keeper: Arc<Mutex<SessionKeeper>>,
    wallet_name: String,
    requester: Requester,
}

impl CFilterProcessor {
    async fn handle(&self, filter: bip157::IndexedFilter) {
        let mut session_keeper = self.session_keeper.lock().await;
        let wallet = match session_keeper.get(&self.wallet_name) {
            Err(e) => {
                eprint!("fail to get wallet from session {e}");
                return;
            }
            Ok(s) => &mut s.wallet,
        };

        let scripts_iter = wallet.btc.scripts_of_interes.iter().map(|s| &s.script);
        if !filter.contains_any(scripts_iter) {
            return;
        }

        let block_hash = filter.block_hash();
        let indexed_block = match self.requester.get_block(block_hash).await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error processing indexed filter: {}", e);
                return;
            }
        };
        let block_height = indexed_block.height;
        let unspent_outputs = indexed_block
            .block
            .txdata
            .iter()
            .flat_map(|tx| {
                wallet
                    .btc
                    .scripts_of_interes
                    .iter()
                    .flat_map(move |derived_script| {
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

        if let Err(e) = wallet.mutate_btc(|btc| {
            let len = unspent_outputs.len();
            btc.insert_utxos(unspent_outputs);
            println!("Saved {} utxos", len);
            Ok(())
        }) {
            eprintln!("fail to insert utxos: {e}");
        }
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    app: AppHandle,
    client: bip157::Client,
    repository: Arc<ChainRepository>,
    cfilter_processor: CFilterProcessor,
) {
    let Client {
        mut info_rx,
        mut warn_rx,
        mut event_rx,
        ..
    } = client;

    loop {
        tokio::select! {
            event = event_rx.recv() => {
            if let Some(event) = event {
                match event {
                Event::FiltersSynced(sync_update) => {
                    println!("Synced to height: {}", sync_update.tip.height);
                    app.emit(
                    EVENT_HEIGHT_UPDATE,
                    SyncHeightUpdateEvent {
                        height: sync_update.tip.height,
                        status: HeightUpdateStatus::Completed,
                    },
                    )
                    .unwrap();
                }
                Event::ChainUpdate(changes) => {
                    let new_height = match changes {
                    BlockHeaderChanges::Connected(header) => {
                        repository
                        .save_block_header(header)
                        .expect("fail to insert block headers");
                        Some(header.height)
                    }
                    BlockHeaderChanges::Reorganized { accepted, .. } => {
                        accepted.last().map(|h| h.height)
                    }
                    BlockHeaderChanges::ForkAdded(_) => None,
                    };
                    if let Some(height) = new_height {
                    app.emit(
                        EVENT_HEIGHT_UPDATE,
                        SyncHeightUpdateEvent {
                        height,
                        status: HeightUpdateStatus::Progress,
                        },
                    )
                    .unwrap();
                    }
                }
                Event::Block(_block) => {}
                Event::IndexedFilter(filter) => {
                    cfilter_processor.handle(filter).await;
                }
                }
            }
            }

            info = info_rx.recv() => {
            if let Some(info) = info {
                match info {
                    bip157::Info::SuccessfulHandshake => {},
                    bip157::Info::ConnectionsMet => {},
                    bip157::Info::Progress(progress) => {
                        app.emit(EVENT_SYNC_PROGRESS, SyncProgressEvent {
                            progress: progress.percentage_complete()
                         }).unwrap();
                    },
                    bip157::Info::BlockReceived(_) => {},
                }
            }
            }

            warn = warn_rx.recv() => {
            if let Some(warn) = warn {
                eprintln!("Bitcoin sync warning: {}", warn);
                app.emit(EVENT_SYNC_WARNING, SyncWarningEvent {
                    msg: warn.to_string()
                }).unwrap();
            }
            }
        }
    }
}
