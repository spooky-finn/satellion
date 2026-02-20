use std::{collections::HashMap, net, str::FromStr, sync::Arc, thread, time::Duration};

use nakamoto::{
    chain::{BlockHash, Transaction},
    client::{Client, Config, Event, Loading, chan, traits::Handle},
    common::{bitcoin::Script, bitcoin_hashes::hex::ToHex, block::Height},
    net::poll::Waker,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{
    btc::{
        address::DerivePath,
        config::Network,
        neutrino::LifecycleState,
        utxo::{BlockHeader, Utxo},
    },
    config::CONFIG,
    session::{SK, SessionKeeper},
};

#[derive(Clone)]
pub struct NeutrinoStarter {
    sk: Arc<Mutex<SessionKeeper>>,
    state: Arc<Mutex<LifecycleState>>,
}

pub struct NodeStartArgs {
    pub birth_height: Height,
}

impl NeutrinoStarter {
    pub fn new(sk: SK) -> Self {
        Self {
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
        // let child_token = cancel_token.child_token();

        let this = self.clone();

        let task = tauri::async_runtime::spawn(async move {
            if let Err(e) = this.run_node(args).await {
                tracing::error!("Neutrino exited: {}", e);
            }
        });

        state.start_for_wallet(wallet_name, task, cancel_token);
        Ok(())
    }

    async fn run_node(&self, args: NodeStartArgs) -> anyhow::Result<()> {
        Neutrino::connect(args.birth_height, CONFIG.bitcoin.network(), self.sk.clone()).await?;
        Ok(())
    }
}

type Reactor = nakamoto::net::poll::Reactor<net::TcpStream>;

pub struct Neutrino {
    pub client: nakamoto::client::Handle<Waker>,
    sk: Arc<Mutex<SessionKeeper>>,
}

impl Neutrino {
    pub async fn connect(
        birth_height: Height,
        network: Network,
        sk: SK,
    ) -> anyhow::Result<Arc<Self>> {
        let cfg = Config {
            network: network.into(),
            listen: vec![], // Don't listen for incoming connections.
            connect: vec![CONFIG.bitcoin.regtest_peer_socket()],
            ..Config::default()
        };

        tracing::info!("starting neutrino on network {:?}", network);

        let client = Client::<Reactor>::new()?;
        let handle = client.handle();
        let client_rx = handle.events();
        let (loading_tx, loading_rx) = chan::unbounded::<Loading>();

        thread::spawn(move || {
            let client_runner = client
                .load(cfg, loading_tx)
                .unwrap_or_else(|e| panic!("fail to load bip 157/158 client: {e}"));
            client_runner.run().expect("client start failed");
        });

        let instance = Arc::new(Self {
            sk: sk.clone(),
            client: handle.clone(),
        });
        let handle = handle.clone();

        let scripts = {
            let mut sk = sk.lock().await;
            let wallet = sk.wallet_mut().map_err(|e| anyhow::anyhow!(e))?;
            let prk = wallet.btc_prk()?;
            wallet.btc.derive_scripts_of_interes(&prk)?
        };

        let script_map: HashMap<Script, DerivePath> = scripts
            .iter()
            .map(|e| {
                // Make convertion from lib vith different versions
                let script = Script::from_str(&e.script.to_hex()).expect("failt to parse script");

                (script, e.derive_path.clone())
            })
            .collect();
        // 1. Grab a handle to the current Tokio runtime
        let tokio_handle = tokio::runtime::Handle::current();

        // 2. Create an unbounded channel to pipe events sequentially to Tokio
        let (sequential_event_tx, mut sequential_event_rx) = tokio::sync::mpsc::unbounded_channel();

        // 3. Spawn a SINGLE Tokio task to process events strictly in order
        let this_for_task = instance.clone();
        let scripts_for_task = script_map.clone();
        tokio_handle.spawn(async move {
            // This loop processes one event at a time, ensuring linear blockchain state
            while let Some(event) = sequential_event_rx.recv().await {
                if let Err(err) = this_for_task
                    .handle_client_event(event, &scripts_for_task)
                    .await
                {
                    tracing::error!("client event error: {:?}", err);
                }
            }
            tracing::info!("Nakamoto sequential event processor shut down");
        });

        thread::spawn(move || {
            // Start scanning from genesis (or your stored height)
            if let Err(e) = handle.rescan(birth_height.., script_map.keys().cloned()) {
                tracing::error!("rescan failed: {:?}", e);
                return;
            }

            loop {
                chan::select! {
                    recv(loading_rx) -> event => {
                        if let Ok(event) = event {
                            tracing::debug!("loading {}", event);
                        } else {
                            break;
                        }
                    }
                }
            }

            loop {
                chan::select! {
                    recv(client_rx) -> event => {
                        if let Ok(e) = event {
                            // 4. Fire the event into our sequential channel
                            if sequential_event_tx.send(e).is_err() {
                                tracing::error!("Event receiver dropped, stopping Nakamoto event loop");
                                break;
                            }

                        } else {
                            tracing::error!("failure receiving client event {:?}", event.err());
                            break;
                        }
                    }

                    recv(chan::after(Duration::from_millis(100))) -> _ => {}
                }
            }
        });

        Ok(instance)
    }

    pub async fn handle_client_event(
        &self,
        event: Event,
        scripts: &HashMap<Script, DerivePath>,
    ) -> anyhow::Result<()> {
        tracing::debug!("{event}");

        match event {
            Event::BlockMatched {
                hash,
                header: _,
                height,
                transactions,
            } => {
                self.process_block_transactions(hash, height, transactions, scripts)
                    .await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn process_block_transactions(
        &self,
        block_hash: BlockHash,
        height: Height,
        transactions: Vec<Transaction>,
        scripts: &HashMap<Script, DerivePath>,
    ) -> anyhow::Result<()> {
        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet_mut()?;

        for tx in transactions {
            let txid = tx.txid();

            for (vout, output) in tx.output.iter().enumerate() {
                if let Some(derive_path) = scripts.get(&output.script_pubkey) {
                    let utxo = Utxo {
                        tx_id: txid,
                        vout: vout as u32,
                        output: output.clone(),
                        derive_path: derive_path.clone(),
                        block: BlockHeader {
                            hash: block_hash,
                            height,
                        },
                    };
                    let out = utxo.out_point();
                    tracing::info!("new utrxo found {}", out);
                    wallet.btc.utxos.insert(out, utxo);
                }
            }

            for input in &tx.input {
                let prev_out = input.previous_output;
                wallet.btc.utxos.remove(&prev_out);
            }
        }

        tracing::debug!("block processing end: utxos {}", wallet.btc.utxos.len());
        Ok(())
    }
}
