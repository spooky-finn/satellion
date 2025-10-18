mod app_state;
mod commands;
mod db;
mod neutrino;
mod repository;
mod schema;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;
use tauri::Manager;

use crate::repository::Repository;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_pool = establish_pool();
    let repository = Repository::new(db_pool.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db_pool.clone())
        .setup(move |app| {
            let state = app.state::<Arc<app_state::AppState>>();

            match neutrino::Neutrino::connect_regtest() {
                Ok(neutrino) => {
                    let (node, client) = (neutrino.node, neutrino.client);

                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = node.run().await {
                            eprintln!("Neutrino node error: {}", e);
                        }
                    });

                    tauri::async_runtime::spawn(neutrino::handle_chain_updates(
                        client,
                        state.inner().clone(),
                        repository,
                    ));
                }
                Err(e) => {
                    eprintln!("Failed to connect to regtest: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::chain_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn establish_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new("blockchain.db");
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Error creating DB pool")
}
