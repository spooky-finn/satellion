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
mod wallet_service;

use crate::{repository::Repository, wallet_service::WalletService};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

const ENABLE_DEVTOOLS: bool = true;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = db::connect();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db.clone())
        .manage(Repository::new(db.clone()))
        .manage(ethereum::provider::new().expect("Failed to create Ethereum client"))
        .manage(Mutex::new(ethereum::TxBuilder::new()))
        .manage(WalletService::new(Repository::new(db.clone())))
        .setup(move |app| {
            #[cfg(debug_assertions)]
            if ENABLE_DEVTOOLS {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
