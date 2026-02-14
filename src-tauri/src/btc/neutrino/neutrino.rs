use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_util::sync::CancellationToken;

use bip157::{
    BlockHash, Builder, Header, HeaderCheckpoint, Network, TrustedPeer,
    chain::{ChainState, IndexedHeader},
};
use bitcoin::{CompactTarget, TxMerkleNode, block::Version};

use crate::{
    btc::{
        DerivedScript,
        address::ScriptHolder,
        neutrino::{
            BoxFutureUnit, EventEmitter, LifecycleState, NodeLifecycle, SyncOrchestrator,
            block_sync_worker::{BlockRequestChannel, BlockSyncWorker},
            node_listener::{NeutrinoClientArgs, run_neutrino_client},
        },
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

pub struct NodeStartArgs {
    pub event_emitter: EventEmitter,
    pub script_rx: mpsc::UnboundedReceiver<DerivedScript>,
    pub last_seen_height: u32,
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
        args: NodeStartArgs,
        wallet_name: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;

        // Case 1: same wallet -> node already running
        if state.is_running_for(&wallet_name) {
            return Ok(());
        }

        // Case 2: different wallet unlocked -> shut down old instance
        state.stop_current();

        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();

        let this = self.clone();

        let task = tauri::async_runtime::spawn(async move {
            if let Err(e) = this.run_node(args, child_token).await {
                tracing::error!("Neutrino exited: {}", e);
            }
        });

        state.start_for_wallet(wallet_name, task, cancel_token);
        Ok(())
    }

    async fn run_node(&self, args: NodeStartArgs, cancel: CancellationToken) -> Result<(), String> {
        let NodeStartArgs {
            event_emitter,
            script_rx: mut scripts_rx,
            last_seen_height,
        } = args;

        let block_header = self
            .repository
            .get_block_header(last_seen_height)
            .map_err(|e| format!("Failed to load block header: {}", e))?;

        let neutrino = Neutrino::connect(block_header)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        let mut sync_orchestrator = SyncOrchestrator::new(
            self.sk.clone(),
            self.repository.clone(),
            event_emitter.clone(),
        );

        let requester = neutrino.client.requester.clone();
        let script_holder = Arc::new(RwLock::new(ScriptHolder::new()));
        let block_downloader = BlockSyncWorker::new(
            requester,
            sync_orchestrator.transmitter(),
            script_holder.clone(),
            event_emitter.clone(),
        );

        let block_req_channel = BlockRequestChannel::default();
        let neutrino_client_args = NeutrinoClientArgs {
            event_emitter,
            sync_event_tx: sync_orchestrator.transmitter(),
            block_req_tx: block_req_channel.tx.clone(),
            script_holder: script_holder.clone(),
        };

        let block_req_rx = block_req_channel.rx;

        let lifecycle = NodeLifecycle::spawn(
            vec![
                (
                    Box::pin(async move {
                        if let Err(e) = neutrino.node.run().await {
                            tracing::error!("Kyoto node err: {}", e);
                        }
                    }) as BoxFutureUnit,
                    "neutrino_node",
                ),
                (
                    Box::pin(run_neutrino_client(neutrino.client, neutrino_client_args))
                        as BoxFutureUnit,
                    "neutrino_client",
                ),
                (
                    Box::pin(async move {
                        while let Some(script) = scripts_rx.recv().await {
                            script_holder.write().await.add(script);
                        }
                    }) as BoxFutureUnit,
                    "scripts_rx",
                ),
                (
                    Box::pin(async move { block_downloader.run(block_req_rx).await })
                        as BoxFutureUnit,
                    "block_donwloader",
                ),
                (
                    Box::pin(async move {
                        if let Err(e) = sync_orchestrator.run().await {
                            tracing::error!("Sync orchestrator failure: {}", e);
                        }
                    }) as BoxFutureUnit,
                    "sync_orchestrator",
                ),
            ],
            cancel.clone(),
        );

        lifecycle.join_all().await;
        Ok(())
    }
}

pub struct Neutrino {
    pub node: bip157::Node,
    pub client: bip157::Client,
}

impl Neutrino {
    pub async fn connect(block_header: Option<db::BlockHeader>) -> Result<Self, String> {
        let (network, trusted_peers) = Self::select_network().await?;
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

        let chain_state: Option<ChainState> = indexed_header
            .map(|ih| {
                ChainState::Checkpoint(HeaderCheckpoint {
                    height: ih.height,
                    hash: ih.block_hash(),
                })
            })
            .or_else(|| {
                if network == Network::Bitcoin {
                    Some(ChainState::Checkpoint(
                        HeaderCheckpoint::taproot_activation(),
                    ))
                } else {
                    None
                }
            });

        tracing::info!(
            "starting neutrino on network {}, chain state {:?}",
            network,
            chain_state
        );

        let mut builder = Builder::new(network);
        if let Some(s) = chain_state {
            builder = builder.chain_state(s);
        }

        let (node, client) = builder
            .required_peers(CONFIG.bitcoin.min_peers)
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
