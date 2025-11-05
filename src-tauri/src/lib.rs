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
    repository::{ChainRepository, WalletRepository},
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
    let db = db::connect();
    let wallet_repository = WalletRepository::new(db.clone());
    let wallet_service = WalletService::new(wallet_repository.clone());

    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::get_available_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            bitcoin::commands::start_node,
            ethereum::commands::eth_chain_info,
            ethereum::commands::eth_get_balance,
            ethereum::commands::eth_prepare_send_tx,
            ethereum::commands::eth_sign_and_send_tx,
            ethereum::commands::eth_verify_address,
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
        .manage(ethereum::provider::new().expect("Failed to create Ethereum client"))
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
