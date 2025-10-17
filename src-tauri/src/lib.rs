mod app_state;
mod commands;
mod neutrino;

use std::sync::Arc;

use crate::neutrino::Neutrino;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state::AppState::new()))
        .setup(move |app| {
            let state = app.state::<Arc<app_state::AppState>>();

            match Neutrino::connect_regtest() {
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
