use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

use bip157::{
    BlockHash, Builder, Header, HeaderCheckpoint, Network, TrustedPeer,
    chain::{ChainState, IndexedHeader},
};
use bitcoin::{CompactTarget, TxMerkleNode, block::Version};

use crate::{
    btc::{
        neutrino::{
            BoxFutureUnit, CompactFilterScanner, EventEmitter, LifecycleState, NodeLifecycle,
            SyncOrchestrator,
            node_listener::{ListenArgs, listen_node},
        },
        sync,
    },
    config::CONFIG,
    db,
    repository::ChainRepository,
    session::{SK, SessionKeeper},
};

static NODE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct NeutrinoStarter {
    sk: Arc<Mutex<SessionKeeper>>,
    repository: ChainRepository,
    state: Arc<Mutex<LifecycleState>>,
}

impl NeutrinoStarter {
    pub fn new(repository: ChainRepository, sk: SK) -> Self {
        Self {
            repository,
            sk,
            state: Arc::new(Mutex::new(LifecycleState::new())),
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
        if state.is_running_for(&wallet_name) {
            tracing::debug!("Neutrino already running for wallet {}", wallet_name);
            return Ok(());
        }

        // Case 2: different wallet unlocked -> shut down old instance
        state.stop_current();

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

        state.start_for_wallet(wallet_name, task, cancel_token);

        Ok(())
    }

    async fn run_node(
        &self,
        event_emitter: EventEmitter,
        last_seen_height: u32,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        let block_header = self
            .repository
            .get_block_header(last_seen_height)
            .map_err(|e| format!("Failed to load block header: {}", e))?;
        tracing::info!("Last seen height {:?}", block_header);

        let neutrino = Neutrino::connect(block_header)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;
        let sync_orchestrator = SyncOrchestrator::new(
            self.sk.clone(),
            self.repository.clone(),
            event_emitter.clone(),
        );

        let (filters_tx, filters_rx) = mpsc::unbounded_channel::<bip157::IndexedFilter>();

        let (mut cf_scanner, listen_args) = {
            let mut session_keeper = self.sk.lock().await;
            let session = session_keeper.take_session()?;

            let sync::Channels {
                scripts_rx,
                sync_event_tx,
                ..
            } = &mut session.wallet.btc.runtime.sync.channels;

            let cf_scanner = CompactFilterScanner::new(
                neutrino.client.requester.clone(),
                scripts_rx.take().ok_or("scripts_rx has been acquired")?,
                filters_rx,
                sync_event_tx.clone(),
            );

            let listen_args = ListenArgs {
                event_emitter,
                client: neutrino.client,
                filters_tx,
                sync_event_tx: sync_event_tx.clone(),
            };

            (cf_scanner, listen_args)
        };

        let lifecycle = NodeLifecycle::spawn(
            vec![
                (
                    Box::pin(Self::run_node_task(neutrino.node)) as BoxFutureUnit,
                    "Neutrino node",
                ),
                (
                    Box::pin(listen_node(listen_args)) as BoxFutureUnit,
                    "Neutrino client",
                ),
                (
                    Box::pin(async move { cf_scanner.run().await }) as BoxFutureUnit,
                    "Compact block filter scanner",
                ),
                (
                    Box::pin(async move {
                        if let Err(e) = sync_orchestrator.run().await {
                            tracing::error!("Sync orchestrator failure: {}", e);
                        }
                    }) as BoxFutureUnit,
                    "Neutrino sync orchestrator",
                ),
            ],
            cancel.clone(),
        );

        lifecycle.join_all().await;
        Ok(())
    }

    async fn run_node_task(node: bip157::Node) {
        if let Err(e) = node.run().await {
            tracing::error!("Kyoto node err: {}", e);
        }
    }
}

pub struct Neutrino {
    pub node: bip157::Node,
    pub client: bip157::Client,
}

impl Neutrino {
    pub async fn connect(block_header: Option<db::BlockHeader>) -> Result<Self, String> {
        let (network, trusted_peers) = Self::select_network().await?;
        tracing::info!("Starting neutrino for network {}", network);

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

        tracing::info!(
            "starting neutrino on network {}, chain state {:?}",
            network,
            chain_state
        );

        let builder = Builder::new(network).required_peers(CONFIG.bitcoin.min_peers);

        let builder = if network == Network::Bitcoin {
            builder.chain_state(chain_state)
        } else {
            builder
        };

        let (node, client) = builder
            .add_peers(trusted_peers)
            .response_timeout(NODE_RESPONSE_TIMEOUT)
            .build();

        Ok(Self { node, client })
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
