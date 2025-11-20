mod app_state;
mod btc;
mod commands;
mod config;
mod db;
mod encryptor;
mod eth;
mod mnemonic;
mod repository;
mod schema;
mod session;
mod wallet_service;

use crate::{
    repository::{ChainRepository, TokenRepository, WalletRepository},
    wallet_service::WalletService,
};
use specta_typescript::Typescript;
use std::sync::Arc;
use tauri::Manager;
use tauri_specta;
use tokio::sync::Mutex;

const ENABLE_DEVTOOLS: bool = true;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    db::initialize();

    let db = db::connect();
    let wallet_repository = WalletRepository::new(db.clone());
    let wallet_service = WalletService::new(wallet_repository.clone());
    let token_repository = TokenRepository::new(db.clone());
    let eth_provider: alloy_provider::DynProvider = eth::new_provider();
    let eth_batch_provider = eth::new_provider_batched();
    let token_manager = eth::token_manager::TokenManager::new(token_repository.clone());
    let token_retriever = eth::token_manager::Erc20Retriever::new(eth_provider.clone());
    let tx_builder = eth::TxBuilder::new(eth_batch_provider);
    let price_feed = eth::PriceFeed::new(eth_provider.clone());

    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::list_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            btc::commands::start_node,
            eth::commands::eth_chain_info,
            eth::commands::eth_get_balance,
            eth::commands::eth_prepare_send_tx,
            eth::commands::eth_sign_and_send_tx,
            eth::commands::eth_verify_address,
            eth::commands::eth_track_token,
            eth::commands::eth_untrack_token,
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
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db.clone())
        .manage(ChainRepository::new(db.clone()))
        .manage(wallet_repository)
        .manage(wallet_service)
        .manage(token_repository)
        .manage(eth_provider.clone())
        .manage(token_manager)
        .manage(token_retriever)
        .manage(price_feed)
        .manage(Mutex::new(tx_builder))
        .manage(tokio::sync::Mutex::new(session::Store::new()))
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
