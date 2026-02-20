use std::{net, str::FromStr, sync::Arc, thread, time::Duration};

use nakamoto::{
    client::{Client, Config, Event, Loading, chan, traits::Handle},
    common::block::Height,
    net::poll::Waker,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{
    btc::{config::Network, neutrino::LifecycleState},
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

    async fn run_node(&self, args: NodeStartArgs) -> Result<(), String> {
        let scripts = {
            let mut sk = self.sk.lock().await;
            let wallet = sk.wallet()?;
            let prk = wallet.btc_prk()?;
            wallet.btc.derive_scripts_of_interes(&prk)?
        };

        let scripts: Vec<nakamoto::common::bitcoin::Script> = scripts
            .iter()
            .map(|e| {
                // Make convertion from lib vith different versions
                nakamoto::common::bitcoin::Script::from_str(&e.script.to_hex_string())
                    .expect("failt to parse script")
            })
            .collect();

        Neutrino::connect(args.birth_height, CONFIG.bitcoin.network(), scripts)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        Ok(())
    }
}

type Reactor = nakamoto::net::poll::Reactor<net::TcpStream>;

pub struct Neutrino {
    pub client: nakamoto::client::Handle<Waker>,
}

impl Neutrino {
    pub async fn connect(
        birth_height: Height,
        network: Network,
        scripts: Vec<nakamoto::common::bitcoin::Script>,
    ) -> Result<Arc<Self>, nakamoto::client::Error> {
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
            client: handle.clone(),
        });
        let this = instance.clone();
        let handle = handle.clone();

        thread::spawn(move || {
            // Start scanning from genesis (or your stored height)
            if let Err(e) = handle.rescan(birth_height.., scripts.clone().into_iter()) {
                tracing::error!("rescan failed: {:?}", e);
                return;
            }

            loop {
                chan::select! {
                    recv(loading_rx) -> event => {
                        if let Ok(event) = event {
                            tracing::debug!("{}",event);
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
                            this.handle_client_event(e);
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

    pub fn handle_client_event(&self, event: Event) {
        tracing::debug!("{event}");
    }
}
