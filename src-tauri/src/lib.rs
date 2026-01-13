mod btc;
mod chain_trait;
mod commands;
mod config;
mod db;
mod encryptor;
mod eth;
mod mnemonic;
mod persistence;
mod repository;
mod schema;
mod session;
mod utils;
mod wallet;
mod wallet_keeper;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::collect_events;
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    btc::neutrino::{NeutrinoStarter, SyncHeightUpdateEvent, SyncProgressEvent, SyncWarningEvent},
    repository::ChainRepository,
    wallet_keeper::WalletKeeper,
};

const ENABLE_DEVTOOLS: bool = false;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .compact()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    db::initialize();

    let db = db::connect();
    let wallet_keeper = WalletKeeper::new();

    let eth_provider = eth::select_provider();
    let eth_batch_provider = eth::new_provider_batched(eth_provider.clone());
    let erc20_retriever = eth::Erc20Retriever::new(eth_provider.clone());
    let tx_builder = eth::TxBuilder::new(eth_batch_provider);
    let price_feed = eth::PriceFeed::new(eth_provider.clone());

    let chain_repository = ChainRepository::new(db.clone());
    let session_keeper = Arc::new(tokio::sync::Mutex::new(session::SessionKeeper::new()));
    let neutrino_starter = NeutrinoStarter::new(chain_repository.clone(), session_keeper.clone());

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
        .events(collect_events![
            SyncHeightUpdateEvent,
            SyncProgressEvent,
            SyncWarningEvent
        ]);

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
        .manage(neutrino_starter)
        .manage(chain_repository)
        .manage(Mutex::new(tx_builder))
        .manage(session_keeper)
        .setup(move |app| {
            #[cfg(debug_assertions)]
            if ENABLE_DEVTOOLS {
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
