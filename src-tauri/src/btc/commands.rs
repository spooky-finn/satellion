use specta::specta;

use crate::{
    btc::wallet::{AddressType, derive_taproot_address},
    config::{CONFIG, Chain},
    repository::wallet_repository::WalletRepositoryImpl,
    session,
    wallet::WalletRepository,
};

#[specta]
#[tauri::command]
pub async fn btc_derive_address(
    wallet_name: String,
    index: u32,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::SessionKeeper>>,
    wallet_repository: tauri::State<'_, WalletRepositoryImpl>,
) -> Result<String, String> {
    let mut session_store = session_store.lock().await;
    let session = session_store.get(&wallet_name)?;
    let btc_session = session
        .get_bitcoin_session()
        .ok_or("Bitcoin session is not initialized")?;
    let net = CONFIG.bitcoin.network();
    let child = derive_taproot_address(&btc_session.xprv, net, AddressType::Receive, index)?;
    wallet_repository.set_last_used_chain(&wallet_name, Chain::Bitcoin)?;
    Ok(child.1.to_string())
}
