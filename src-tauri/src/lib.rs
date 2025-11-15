mod app_state;
mod bitcoin;
mod commands;
mod config;
mod db;
mod envelope_encryption;
mod ethereum;
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
    db::initialize_database();

    let db = db::connect();
    let wallet_repository = WalletRepository::new(db.clone());
    let wallet_service = WalletService::new(wallet_repository.clone());
    let token_repository = TokenRepository::new(db.clone());
    let eth_provider = ethereum::new_provider();

    let token_manager =
        ethereum::token_manager::TokenManager::new(eth_provider.clone(), token_repository.clone());

    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::list_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            bitcoin::commands::start_node,
            ethereum::commands::eth_chain_info,
            ethereum::commands::eth_get_balance,
            ethereum::commands::eth_prepare_send_tx,
            ethereum::commands::eth_sign_and_send_tx,
            ethereum::commands::eth_verify_address,
            ethereum::commands::eth_track_token,
            ethereum::commands::eth_untrack_token,
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
        .manage(eth_provider)
        .manage(token_manager)
        .manage(Mutex::new(ethereum::TxBuilder::new()))
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
