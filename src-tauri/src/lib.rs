mod app_state;
mod commands;
mod neutrino;

use crate::neutrino::Neutrino;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let block_height = Arc::new(RwLock::new(None));
    let height_clone = block_height.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state::AppState::new())
        .setup(move |_app| {
            match Neutrino::connect_regtest() {
                Ok(neutrino) => {
                    let (node, client) = (neutrino.node, neutrino.client);

                    // Spawn the node to run in the background
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = node.run().await {
                            eprintln!("Neutrino node error: {}", e);
                        }
                    });

                    // Spawn the event handler
                    tauri::async_runtime::spawn(neutrino::handle_events(client, height_clone));
                }
                Err(e) => {
                    eprintln!("Failed to connect to regtest: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_block_height
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
