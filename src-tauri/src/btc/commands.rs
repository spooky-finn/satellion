use shush_rs::ExposeSecret;
use specta::specta;

use crate::{
    btc::{self, wallet::AddressPurpose},
    config::{CONFIG, Chain},
    session::AppSession,
    wallet_keeper::WalletKeeper,
};

#[specta]
#[tauri::command]
pub async fn btc_derive_address(
    wallet_name: String,
    label: String,
    index: u32,
    session_keeper: tauri::State<'_, AppSession>,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<String, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;
    let purpose = AddressPurpose::Receive;
    if !session
        .wallet
        .btc
        .is_deriviation_index_available(purpose.clone(), index)
    {
        return Err(format!("Deriviation index {} already occupied", index));
    }

    let prk = btc::wallet::derive_prk(
        &session.wallet.mnemonic.expose_secret(),
        &session.passphrase.expose_secret(),
    )?;
    let child = session.wallet.btc.derive_child(
        &prk.xpriv,
        CONFIG.bitcoin.network(),
        purpose.clone(),
        index,
    )?;

    session.wallet.last_used_chain = Chain::Bitcoin;
    session.wallet.btc.add_child(label, purpose, index);

    wallet_keeper.save_wallet(session)?;
    Ok(child.1.to_string())
}

#[specta]
#[tauri::command]
pub async fn btc_unoccupied_deriviation_index(
    wallet_name: String,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<u32, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;
    Ok(session
        .wallet
        .btc
        .unoccupied_deriviation_index(AddressPurpose::Receive))
}
