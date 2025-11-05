use crate::app_state::AppState;
use crate::bitcoin;
use crate::repository::ChainRepository;
use std::sync::Arc;

#[tauri::command]
pub async fn start_node(
    state: tauri::State<'_, Arc<AppState>>,
    repository: tauri::State<'_, ChainRepository>,
) -> Result<(), String> {
    // Load block headers from the database using the repository
    let block_headers = match repository.load_block_headers(10) {
        Ok(headers) => headers,
        Err(e) => return Err(format!("Failed to load block headers: {}", e)),
    };

    // Try to connect the regtest neutrino node with headers
    let neutrino = match bitcoin::neutrino::Neutrino::connect_regtest(block_headers) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to connect to regtest: {}", e);
            return Err(e);
        }
    };

    let node = neutrino.node;
    let client = neutrino.client;
    let app_state = state.inner().clone();
    let repository = Arc::new(repository.inner().clone());

    tauri::async_runtime::spawn(async move {
        if let Err(e) = node.run().await {
            eprintln!("Neutrino node error: {}", e);
        }
    });

    tauri::async_runtime::spawn(bitcoin::neutrino::handle_chain_updates(
        client, app_state, repository,
    ));

    Ok(())
}
