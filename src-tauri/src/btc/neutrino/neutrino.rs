use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use bip157::{
    BlockHash, Builder, Client, Event, Header, HeaderCheckpoint, Network, Requester, TrustedPeer,
    chain::{BlockHeaderChanges, ChainState, IndexedHeader},
};
use bitcoin::{CompactTarget, TxMerkleNode, block::Version};
use tauri::async_runtime::JoinHandle;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::{
    btc::{
        neutrino::{EventEmitter, HeightUpdateStatus},
        utxo::UTxO,
    },
    config::CONFIG,
    db,
    repository::ChainRepository,
    session::{SK, SessionKeeper},
    utils::Throttler,
};

#[derive(Clone)]
pub struct NeutrinoStarter {
    sk: Arc<Mutex<SessionKeeper>>,
    repository: ChainRepository,

    state: Arc<Mutex<NeutrinoState>>,
}

struct NeutrinoState {
    running_for_wallet: Option<String>,
    cancel_token: Option<CancellationToken>,
    task: Option<JoinHandle<()>>,
}

impl NeutrinoStarter {
    pub fn new(repository: ChainRepository, sk: SK) -> Self {
        Self {
            repository,
            sk,
            state: Arc::new(Mutex::new(NeutrinoState {
                running_for_wallet: None,
                cancel_token: None,
                task: None,
            })),
        }
    }

    pub async fn request_node_start(
        &self,
        event_emitter: EventEmitter,
        wallet_name: String,
        last_seen_height: u32,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        // Case 1: same wallet -> node already running
        if state.running_for_wallet.as_deref() == Some(&wallet_name) {
            tracing::debug!("Neutrino already running for wallet {}", wallet_name);
            return Ok(());
        }
        // Case 2: different wallet unlocked -> shut down old instance
        if let Some(token) = state.cancel_token.take() {
            tracing::info!("Stopping neutrino for previous walletg");
            token.cancel();
        }

        if let Some(task) = state.task.take() {
            task.abort(); // defensive
        }

        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();

        let this = self.clone();

        let task = tauri::async_runtime::spawn(async move {
            if let Err(e) = this
                .run_node(event_emitter, last_seen_height, child_token)
                .await
            {
                tracing::error!("Neutrino exited: {}", e);
            }
        });

        state.running_for_wallet = Some(wallet_name);
        state.cancel_token = Some(cancel_token);
        state.task = Some(task);

        Ok(())
    }

    async fn run_node(
        &self,
        event_emitter: EventEmitter,
        last_seen_height: u32,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        let (network, trusted_peers) = NeutrinoStarter::select_network().await?;
        info!("Starting neutrino for network {}", network);

        let block_header = self
            .repository
            .get_block_header(last_seen_height)
            .map_err(|e| format!("Failed to load block header: {}", e))?;
        info!("Last seen height {:?}", block_header,);

        let neutrino = Neutrino::connect(network, trusted_peers, block_header)
            .map_err(|e| format!("Failed to connect to regtest: {}", e))?;

        let node = neutrino.node;
        let client = neutrino.client;
        let repository = Arc::new(self.repository.clone());

        let cfilter_processor = CFilterScanner {
            sk: self.sk.clone(),
            requester: client.requester.clone(),
        };

        let node_cancel = cancel.clone();
        let events_cancel = cancel.clone();

        let node_task = tauri::async_runtime::spawn(async move {
            tokio::select! {
                res = node.run() => {
                    if let Err(e) = res {
                        tracing::error!("Neutrino start err: {}", e);
                    }
                }
                _ = node_cancel.cancelled() => {
                    tracing::info!("Neutrino node canceled");
                }
            }
        });

        let event_rx_task = tauri::async_runtime::spawn(async move {
            tokio::select! {
                _ = handle_chain_updates(
                    event_emitter,
                    client,
                    repository,
                    cfilter_processor,
                ) => {}
                _ = events_cancel.cancelled() => {
                    tracing::info!("Neutrino event loop canceled");
                }
            }
        });

        cancel.cancelled().await; // Pause this function until someone else calls cancel()
        node_task.abort();
        event_rx_task.abort();

        Ok(())
    }

    async fn select_network() -> Result<(Network, Vec<TrustedPeer>), String> {
        if CONFIG.bitcoin.regtest {
            let socket = CONFIG.bitcoin.regtest_peer_socket();
            let peer = TrustedPeer::from_socket_addr(socket);
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
        block_header: Option<db::BlockHeader>,
    ) -> Result<Self, String> {
        let indexed_header = block_header.map(|h| IndexedHeader {
            height: h.height as u32,
            header: Header {
                merkle_root: TxMerkleNode::from_str(&h.merkle_root).unwrap(),
                prev_blockhash: BlockHash::from_str(&h.prev_blockhash).unwrap(),
                time: h.time as u32,
                version: Version::from_consensus(h.version),
                bits: CompactTarget::from_consensus(h.bits as u32),
                nonce: h.nonce as u32,
            },
        });
        let chain_state = indexed_header
            .map(|ih| {
                // Todo: investigate how to migrate to snapshot state initialization
                ChainState::Checkpoint(HeaderCheckpoint {
                    height: ih.height,
                    hash: ih.block_hash(),
                })
            })
            .unwrap_or_else(|| ChainState::Checkpoint(HeaderCheckpoint::taproot_activation()));

        info!(
            "starting neutrino on network {}, chain state {:?}",
            network, chain_state
        );
        let (node, client) = Builder::new(network)
            .required_peers(CONFIG.bitcoin.min_peers)
            .chain_state(chain_state)
            .add_peers(trusted_peers)
            .response_timeout(Duration::from_secs(10))
            .build();
        Ok(Self { node, client })
    }
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
        wallet.btc.insert_utxos(utxos);
    }

    pub async fn update_scanner_height(&self, height: u32) -> Result<(), String> {
        let mut sk = self.sk.lock().await;
        let wallet = match sk.take_session() {
            Ok(s) => &mut s.wallet,
            Err(e) => return Err(e),
        };
        wallet.btc.cfilter_scanner_height = height;
        wallet
            .persist()
            .map_err(|e| format!("Bitcoin sync: fail to save wallet: {}", e))
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    event_emitter: EventEmitter,
    client: bip157::Client,
    repository: Arc<ChainRepository>,
    cfilter_processor: CFilterScanner,
) {
    let Client {
        mut info_rx,
        mut warn_rx,
        mut event_rx,
        requester,
    } = client;
    // let mut block_counter = 0;
    let now = Instant::now();
    let mut progress_throttler = Throttler::new(Duration::from_secs(1));
    let mut height_throttler = Throttler::new(Duration::from_secs(1));
    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    match event {
                    Event::FiltersSynced(sync_update) => {
                        debug!("Bitcoin sync: cfilter synced: height {}", sync_update.tip.height);
                        // Request information from the node
                        match requester.broadcast_min_feerate().await {
                            Ok(fee) => tracing::info!("Minimum transaction broadcast fee rate: {:#}", fee),
                            Err(e) => error!("Failed to get broadcast min feerate: {}", e),
                        }
                        let sync_time = now.elapsed().as_secs_f32();
                        tracing::info!("Total sync time: {sync_time} seconds");
                        match requester.average_fee_rate(sync_update.tip().hash).await {
                            Ok(avg_fee_rate) => tracing::info!("Last block average fee rate: {:#}", avg_fee_rate),
                            Err(e) => error!("Failed to get average fee rate: {}", e),
                        }

                        event_emitter.height_updated(sync_update.tip.height, HeightUpdateStatus::Completed);
                        if let Err(e) = cfilter_processor.update_scanner_height(sync_update.tip.height).await {
                            error!(e);
                        }
                    }
                    Event::ChainUpdate(changes) => {
                        let new_height = match changes {
                            BlockHeaderChanges::Connected(header) => {
                                if let Err(err) = repository.insert_block_header(&header) &&
                                    !err.to_string().contains("UNIQUE constraint failed") {
                                        error!("Bitcoin sync: warning: failed to insert block header: {}", err);
                                    }
                                Some(header.height)
                            }
                            BlockHeaderChanges::Reorganized { accepted, .. } => {
                                accepted.last().map(|h| h.height)
                            }
                            BlockHeaderChanges::ForkAdded(_) => None,
                        };
                        if let Some(height) = new_height && height_throttler.should_emit() {
                            event_emitter.height_updated(height, HeightUpdateStatus::Progress);
                        }
                    }
                    Event::Block(_block) => {
                        debug!("Bitcoin sync: block {}", _block.height);
                    }
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
                                let pct = progress.percentage_complete();
                                if pct != 0.0  && progress_throttler.should_emit() {
                                    debug!("Block filter download progress {}", pct);
                                    event_emitter.cf_sync_progress(pct)
                                }
                        },
                        bip157::Info::BlockReceived(_) => {},
                        }
                }
            }

            warn = warn_rx.recv() => {
                if let Some(warn) = warn {
                    warn!("Bitcoin sync: warning: {}", warn);
                    event_emitter.node_warning(warn.to_string())
                }
            }
        }
    }
}
