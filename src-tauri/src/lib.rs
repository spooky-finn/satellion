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
mod wallet_storage;

use crate::config::Config;
use crate::repository::Repository;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;
use tauri::Manager;

const ENABLE_DEVTOOLS: bool = true;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_pool = connect_db();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db_pool.clone())
        .manage(Repository::new(db_pool.clone()))
        .manage(ethereum::provider::new().expect("Failed to create Ethereum client"))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn connect_db() -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(
        Config::db_path()
            .expect("Failed to get DB path")
            .to_string_lossy()
            .to_string(),
    );
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Error creating DB pool")
}
