use crate::repository::{AvailableWallet, Repository};
use crate::{app_state::AppState, db::BlockHeader, schema};
use crate::{mnemonic, wallet_storage};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct SyncStatus {
    pub height: u32,
    pub sync_completed: bool,
}

#[tauri::command]
pub async fn chain_status(
    state: tauri::State<'_, Arc<AppState>>,
    db_pool: tauri::State<'_, crate::db::Pool>,
) -> Result<SyncStatus, String> {
    let mut conn = db_pool.get().expect("Error getting connection from pool");

    let sync_completed = state
        .sync_completed
        .lock()
        .map_err(|_| "Failed to lock sync completed".to_string())?;

    let last_block = schema::bitcoin_block_headers::table
        .select(schema::bitcoin_block_headers::all_columns)
        .order(schema::bitcoin_block_headers::height.desc())
        .first::<BlockHeader>(&mut conn)
        .map_err(|_| "Error getting last block height".to_string())?;

    Ok(SyncStatus {
        height: last_block.height as u32,
        sync_completed: *sync_completed,
    })
}

#[tauri::command]
pub async fn generate_mnemonic() -> Result<String, String> {
    let mnemonic = mnemonic::generate_random(12);
    Ok(mnemonic.join(" "))
}

#[tauri::command]
pub async fn create_wallet(
    mnemonic: String,
    passphrase: String,
    name: String,
    repository: tauri::State<'_, Repository>,
) -> Result<(), String> {
    let wallet = wallet_storage::create_encrypted_wallet(mnemonic, passphrase, name)
        .map_err(|e| e.to_string())?;
    repository
        .insert_wallet(wallet)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_available_wallets(
    repository: tauri::State<'_, Repository>,
) -> Result<Vec<AvailableWallet>, String> {
    let wallets_info = repository.available_wallets().map_err(|e| e.to_string())?;
    Ok(wallets_info)
}

#[tauri::command]
pub async fn unlock_wallet(
    wallet_id: i32,
    passphrase: String,
    repository: tauri::State<'_, Repository>,
) -> Result<String, String> {
    let wallet = repository
        .get_wallet_by_id(wallet_id)
        .map_err(|e| format!("Failed to get wallet: {}", e))?;
    let decrypted = wallet_storage::decrypt_wallet(&wallet, passphrase)?;
    Ok(decrypted.mnemonic)
}
