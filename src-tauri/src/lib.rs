mod app_state;
mod commands;
mod neutrino;

use crate::neutrino::Neutrino;
use sqlx::{Connection, SqliteConnection};
use std::{fs::File, path::Path, sync::Arc};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let db_conn = open_database().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .manage(db_conn)
        .setup(move |app| {
            let state = app.state::<Arc<app_state::AppState>>();

            match Neutrino::connect_regtest() {
                Ok(neutrino) => {
                    let (node, event_rx) = (neutrino.node, neutrino.client.event_rx);

                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = node.run().await {
                            eprintln!("Neutrino node error: {}", e);
                        }
                    });

                    tauri::async_runtime::spawn(neutrino::handle_chain_updates(
                        event_rx,
                        state.inner().clone(),
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

async fn open_database() -> SqliteConnection {
    let url = "sqlite://./blockchain.db";
    if !Path::new("./blockchain.db").exists() {
        File::create("./blockchain.db").expect("Failed to create database");
    }

    SqliteConnection::connect(url)
        .await
        .expect("Failed to connect to database")
}
