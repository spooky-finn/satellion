pub mod btc;
pub mod chain_trait;
pub mod codegen;
pub mod commands;
pub mod config;
pub mod db;
pub mod encryptor;
pub mod eth;
pub mod event_emitter;
pub mod mnemonic;
pub mod persistence;
pub mod repository;
pub mod schema;
pub mod session;
pub mod system;
pub mod utils;
pub mod wallet;
pub mod wallet_keeper;

pub use core::fmt;
pub use std::{sync::Arc, time::Duration};

use tauri::{Listener, Manager};
use tokio::sync::Mutex;

use crate::{
    config::Config,
    event_emitter::{EventEmitter, EventEmitterTrait},
    session::SessionKeeper,
    wallet_keeper::WalletKeeper,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::tracing::init();
    db::initialize();

    let db = db::connect();
    let wallet_keeper = WalletKeeper::default();
    let config = Config::new();
    let eth_provider = eth::select_provider(config.eth.clone());
    let eth_batch_provider = eth::new_provider_batched(eth_provider.clone());
    let erc20_retriever = eth::Erc20Retriever::new(eth_provider.clone());
    let tx_builder = eth::TxBuilder::new(eth_batch_provider);
    let price_feed = eth::PriceFeed::new(eth_provider.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(db.clone())
        .manage(wallet_keeper)
        .manage(eth_provider.clone())
        .manage(erc20_retriever)
        .manage(price_feed)
        .manage(config)
        .manage(Mutex::new(tx_builder))
        .setup(move |app| {
            let event_emitter = EventEmitter::new(app.handle().clone());
            let sk = SessionKeeper::new(Some(event_emitter.clone()), Some(Duration::from_mins(1)));
            app.manage(sk.clone());

            system::session_monitor::init(app.handle());
            let app_handle = app.handle();
            setup_session_listeners(app_handle, sk, event_emitter.into());

            #[cfg(debug_assertions)]
            if enable_devtools() {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(codegen::handlers().invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn enable_devtools() -> bool {
    std::env::var("DEVTOOLS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn setup_session_listeners(
    app_handle: &tauri::AppHandle,
    sk: Arc<Mutex<SessionKeeper>>,
    event_emitter: Arc<EventEmitter>,
) {
    // Listener for session lock
    {
        let sk = sk.clone();
        app_handle.listen(
            system::session_monitor::SYS_SESSION_LOCKED_EVENT,
            move |_| {
                let sk = sk.clone();
                tauri::async_runtime::spawn(async move {
                    let mut sk = sk.lock().await;
                    sk.soft_terminate();
                });
            },
        );
    }

    // Listener for session unlock
    {
        let sk = sk.clone();
        let em = event_emitter.clone();
        app_handle.listen(
            system::session_monitor::SYS_SESSION_UNLOCKED_EVENT,
            move |_| {
                let sk = sk.clone();
                let emmiter = em.clone();
                tauri::async_runtime::spawn(async move {
                    let sk = sk.lock().await;
                    // If no session exist just emit event to redirect UI
                    if !sk.has_session() {
                        emmiter.session_expired();
                    }
                });
            },
        );
    }
}
