use crate::mnemonic;
use crate::repository::Repository;
use crate::{app_state::AppState, db::BlockHeader, schema};
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
pub async fn wallet_exists(repository: tauri::State<'_, Repository>) -> Result<bool, String> {
    let exist = repository.wallet_exist();
    if exist.is_err() {
        return Err("wallet dose not exist".to_string());
    }
    Ok(exist.unwrap())
}

#[tauri::command]
pub async fn generate_mnemonic() -> Result<String, String> {
    let mnemonic = mnemonic::generate_random(12);
    Ok(mnemonic.join(" "))
}

#[tauri::command]
pub async fn save_mnemonic(
    mnemonic: String,
    passphrase: String,
    name: String,
    repository: tauri::State<'_, Repository>,
) -> Result<(), String> {
    let private_key = mnemonic + " " + &passphrase;
    let result = repository.save_private_key(private_key, name);
    if result.is_err() {
        return Err(result.err().unwrap().to_string());
    }
    Ok(())
}
