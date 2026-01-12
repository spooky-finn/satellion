use std::{collections::HashMap, net::SocketAddrV4, str::FromStr, sync::Arc, time::Duration};

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
use tracing::{debug, error, info, warn};

use crate::{
    btc::utxo::UTxO,
    config::CONFIG,
    db::BlockHeader,
    repository::ChainRepository,
    session::{SK, SessionKeeper},
};

const REGTEST_PEER: &str = "127.0.0.1:18444";

pub const EVENT_HEIGHT_UPDATE: &str = "btc_sync";
pub const EVENT_SYNC_PROGRESS: &str = "btc_sync_progress";
pub const EVENT_SYNC_WARNING: &str = "btc_sync_warning";

#[derive(Clone)]
pub struct NeutrinoStarter {
    sk: Arc<Mutex<SessionKeeper>>,
    repository: ChainRepository,
}

impl NeutrinoStarter {
    pub fn new(repository: ChainRepository, sk: SK) -> Self {
        Self { repository, sk }
    }

    pub async fn sync(&self, app: AppHandle, last_seen_height: u32) -> Result<(), String> {
        let block_headers = self
            .repository
            .get_block_headers(last_seen_height, 10)
            .map_err(|e| format!("Failed to load block headers: {}", e))?;
        debug!("Last seen height {}", last_seen_height);
        debug!(
            "is block headers contains last checkd wallet {}",
            block_headers
                .iter()
                .any(|each| each.height as u32 == last_seen_height),
        );

        let (network, trusted_peers) = NeutrinoStarter::select_network().await?;
        info!("Starting neutrino for network {}", network);
        let neutrino = Neutrino::connect(network, trusted_peers, block_headers)
            .map_err(|e| format!("Failed to connect to regtest: {}", e))?;

        let node = neutrino.node;
        let client = neutrino.client;
        let repository = Arc::new(self.repository.clone());

        tauri::async_runtime::spawn(async move {
            if let Err(e) = node.run().await {
                error!("Neutrino err: {}", e);
            }
        });

        let cfilter_processor = CFilterScanner {
            sk: self.sk.clone(),
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

pub struct CFilterScanner {
    sk: SK,
    requester: Requester,
}

impl CFilterScanner {
    async fn handle(&self, filter: bip157::IndexedFilter) {
        let scripts_of_interes = {
            let mut sk = self.sk.lock().await;
            let wallet = match sk.take_session() {
                Ok(s) => &s.wallet,
                Err(_) => return,
            };
            wallet.btc.runtime.scripts_of_interes.clone()
        };
        let script_map: HashMap<_, _> = scripts_of_interes
            .into_iter()
            .map(|s| (s.script.clone(), s.derive_path))
            .collect();
        if !filter.contains_any(script_map.keys()) {
            return;
        }

        let block_hash = filter.block_hash();
        let indexed_block = match self.requester.get_block(block_hash).await {
            Ok(b) => b,
            Err(e) => {
                error!("Neutrino requester: get block: {}", e);
                return;
            }
        };
        let block_height = indexed_block.height;
        let mut utxos: Vec<UTxO> = vec![];

        for tx in &indexed_block.block.txdata {
            for (vout, output) in tx.output.iter().enumerate() {
                if let Some(derive_path) = script_map.get(&output.script_pubkey) {
                    utxos.push(UTxO {
                        tx_id: tx.compute_wtxid(),
                        output: output.clone(),
                        vout,
                        derive_path: derive_path.clone(),
                        block: crate::btc::utxo::BlockHeader {
                            hash: block_hash,
                            height: block_height,
                        },
                    });
                }
            }
        }

        let mut sk = self.sk.lock().await;
        let wallet = match sk.take_session() {
            Ok(s) => &mut s.wallet,
            Err(_) => return,
        };
        wallet.btc.insert_utxos(block_height, utxos);
    }

    pub async fn save(&self) -> Result<(), String> {
        let mut sk = self.sk.lock().await;
        let wallet = match sk.take_session() {
            Ok(s) => &mut s.wallet,
            Err(e) => return Err(e),
        };
        wallet
            .persist()
            .map_err(|e| format!("Bitcoin sync: fail to save wallet: {}", e))
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    app: AppHandle,
    client: bip157::Client,
    repository: Arc<ChainRepository>,
    cfilter_processor: CFilterScanner,
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
                        debug!("Bitcoin sync: height {}", sync_update.tip.height);
                        app.emit(
                            EVENT_HEIGHT_UPDATE,
                            SyncHeightUpdateEvent {
                                height: sync_update.tip.height,
                                status: HeightUpdateStatus::Completed,
                            },
                        )
                        .unwrap();
                        if let Err(e) = cfilter_processor.save().await {
                            error!(e);
                        }
                    }
                    Event::ChainUpdate(changes) => {
                        let new_height = match changes {
                            BlockHeaderChanges::Connected(header) => {
                            debug!("Bitcoin sync: chain update {}", header.height);
                        if let Err(err) = repository.save_block_header(header) {
                            error!("Bitcoin sync: warning: failed to insert block header: {}", err);
                        }
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
                    Event::Block(_block) => {
                        debug!("Bitcoin sync: block {}", _block.height);
                    }
                    Event::IndexedFilter(filter) => {
                        debug!("Bitcoin sync: cfilter: block {}", filter.height());
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
                            debug!("Bitcoin sync: chain headers: progress {}", progress.percentage_complete());
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
                    warn!("Bitcoin sync: warning: {}", warn);
                    app.emit(EVENT_SYNC_WARNING, SyncWarningEvent {
                        msg: warn.to_string()
                    }).unwrap();
                }
            }
        }
    }
}
