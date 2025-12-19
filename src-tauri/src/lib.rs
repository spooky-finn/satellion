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
mod wallet;
mod wallet_service;

use std::sync::Arc;

use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta;
use tokio::sync::Mutex;

use crate::{
    repository::{ChainRepository, wallet_repository::WalletRepositoryImpl},
    wallet::WalletRepository,
    wallet_service::WalletService,
};

const ENABLE_DEVTOOLS: bool = false;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    db::initialize();

    let db = db::connect();
    let wallet_repository_impl = WalletRepositoryImpl::new();
    let wallet_repository = Arc::new(wallet_repository_impl.clone());
    let wallet_service = WalletService::new(wallet_repository.clone());

    let eth_provider: alloy_provider::DynProvider = eth::select_provider();
    let eth_batch_provider = eth::new_provider_batched(eth_provider.clone());
    let token_retriever = eth::erc20_retriver::Erc20Retriever::new(eth_provider.clone());
    let tx_builder = eth::TxBuilder::new(eth_batch_provider);
    let price_feed = eth::PriceFeed::new(eth_provider.clone());

    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::list_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            commands::get_config,
            btc::commands::start_node,
            btc::commands::btc_derive_address,
            eth::commands::eth_chain_info,
            eth::commands::eth_get_balance,
            eth::commands::eth_prepare_send_tx,
            eth::commands::eth_sign_and_send_tx,
            eth::commands::eth_verify_address,
            eth::commands::eth_track_token,
            eth::commands::eth_untrack_token,
            eth::commands::eth_anvil_set_initial_balances,
        ])
        .constant("MIN_PASSPHRASE_LEN", config::MIN_PASSPHRASE_LEN);

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
        .manage(wallet_repository_impl.clone())
        .manage(wallet_service)
        .manage(eth_provider.clone())
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
