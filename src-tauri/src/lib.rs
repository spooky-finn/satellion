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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_pool = connect_db();
    // let repository = Repository::new(db_pool.clone());
    // let block_headers = repository.load_block_headers(10).unwrap();
    let ethereum_client = ethereum::client::new_client().expect("Failed to create Ethereum client");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db_pool.clone())
        .manage(Repository::new(db_pool.clone()))
        .manage(ethereum_client)
        .setup(move |app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            // let state = app.state::<Arc<app_state::AppState>>();
            // match bitcoin::Neutrino::connect_regtest(block_headers) {
            //     Ok(neutrino) => {
            //         let (node, client) = (neutrino.node, neutrino.client);

            //         tauri::async_runtime::spawn(async move {
            //             if let Err(e) = node.run().await {
            //                 eprintln!("Neutrino node error: {}", e);
            //             }
            //         });

            //         tauri::async_runtime::spawn(bitcoin::handle_chain_updates(
            //             client,
            //             state.inner().clone(),
            //             repository,
            //         ));
            //     }
            //     Err(e) => {
            //         eprintln!("Failed to connect to regtest: {}", e);
            //     }
            // }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::chain_status,
            commands::get_available_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            ethereum::commands::eth_chain_info,
            ethereum::commands::eth_get_balance,
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
