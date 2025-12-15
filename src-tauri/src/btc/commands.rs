use crate::btc;
use crate::btc::wallet::{AddressType, derive_taproot_address};
use crate::config::{CONFIG, Chain};
use crate::repository::{ChainRepository, WalletRepository};
use crate::{app_state::AppState, session};
use specta::specta;
use std::sync::Arc;

#[specta]
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
    let neutrino = match btc::neutrino::Neutrino::connect_regtest(block_headers) {
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

    tauri::async_runtime::spawn(btc::neutrino::handle_chain_updates(
        client, app_state, repository,
    ));

    Ok(())
}

#[specta]
#[tauri::command]
pub async fn btc_derive_address(
    wallet_id: i32,
    index: u32,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
    wallet_repository: tauri::State<'_, WalletRepository>,
) -> Result<String, String> {
    let mut session_store = session_store.lock().await;
    let session = session_store.get(wallet_id).ok_or("Session not found")?;
    let btc_session = session
        .get_bitcoin_session()
        .ok_or("Ethereum session is not initialized")?;
    let net = CONFIG.bitcoin.network();
    let child = derive_taproot_address(&btc_session.xprv, net, AddressType::Receive, index)?;
    wallet_repository.set_last_used_chain(wallet_id, Chain::Bitcoin)?;
    Ok(child.1.to_string())
}
