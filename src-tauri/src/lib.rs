pub mod btc;
pub mod chain_trait;
pub mod commands;
pub mod config;
pub mod db;
pub mod encryptor;
pub mod eth;
pub mod mnemonic;
pub mod persistence;
pub mod repository;
pub mod schema;
pub mod session;
pub mod system;
pub mod utils;
pub mod wallet;
pub mod wallet_keeper;

use std::{sync::Arc, time::Duration};

use specta_typescript::Typescript;
use tauri::{Listener, Manager};
use tokio::sync::Mutex;

use crate::{
    btc::neutrino::{EventEmitter, NeutrinoStarter},
    repository::ChainRepository,
    session::SessionKeeper,
    wallet_keeper::WalletKeeper,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::tracing::init();
    db::initialize();

    let db = db::connect();
    let wallet_keeper = WalletKeeper::new();

    let eth_provider = eth::select_provider();
    let eth_batch_provider = eth::new_provider_batched(eth_provider.clone());
    let erc20_retriever = eth::Erc20Retriever::new(eth_provider.clone());
    let tx_builder = eth::TxBuilder::new(eth_batch_provider);
    let price_feed = eth::PriceFeed::new(eth_provider.clone());

    let chain_repository = ChainRepository::new(db.clone());

    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::list_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            commands::get_config,
            commands::chain_switch_event,
            commands::price_feed,
            btc::commands::btc_derive_address,
            btc::commands::btc_unoccupied_deriviation_index,
            btc::commands::btc_list_derived_addresess,
            btc::commands::btc_list_utxos,
            eth::commands::eth_chain_info,
            eth::commands::eth_get_balance,
            eth::commands::eth_prepare_send_tx,
            eth::commands::eth_sign_and_send_tx,
            eth::commands::eth_verify_address,
            eth::commands::eth_track_token,
            eth::commands::eth_untrack_token,
            eth::commands::eth_anvil_set_initial_balances,
        ])
        .constant("MIN_PASSPHRASE_LEN", config::MIN_PASSPHRASE_LEN)
        .events(btc::neutrino::list_events());

    #[cfg(debug_assertions)]
    builder
        .export(
            Typescript::default().formatter(specta_typescript::formatter::prettier),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(db.clone())
        .manage(wallet_keeper)
        .manage(eth_provider.clone())
        .manage(erc20_retriever)
        .manage(price_feed)
        .manage(chain_repository.clone())
        .manage(Mutex::new(tx_builder))
        .setup(move |app| {
            let event_emitter = EventEmitter::new(app.handle().clone());
            let sk = SessionKeeper::new(Some(event_emitter.clone()), Some(Duration::from_mins(1)));
            let neutrino_starter = NeutrinoStarter::new(sk.clone());

            app.manage(sk.clone());
            app.manage(neutrino_starter);

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
        .invoke_handler(builder.invoke_handler())
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
