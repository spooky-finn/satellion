use crate::config::Config;
use crate::repository::{AvailableWallet, WalletRepository};
use crate::wallet_service::WalletService;
use crate::{
    app_state::AppState,
    db::{BlockHeader, Chain, Token},
    schema,
};
use crate::{bitcoin, session};
use crate::{ethereum, mnemonic};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::Serialize;
use specta::{Type, specta};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Type, Serialize)]
pub struct SyncStatus {
    pub height: u32,
    pub sync_completed: bool,
}

#[specta]
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

#[specta]
#[tauri::command]
pub async fn generate_mnemonic() -> Result<String, String> {
    Ok(mnemonic::new())
}

#[specta]
#[tauri::command]
pub async fn create_wallet(
    mnemonic: String,
    passphrase: String,
    name: String,
    storage: tauri::State<'_, WalletService>,
) -> Result<bool, String> {
    storage.create(mnemonic, passphrase, name)?;
    Ok(true)
}

#[specta]
#[tauri::command]
pub async fn get_available_wallets(
    repository: tauri::State<'_, WalletRepository>,
) -> Result<Vec<AvailableWallet>, String> {
    let wallets_info = repository.list().map_err(|e| e.to_string())?;
    Ok(wallets_info)
}

#[derive(Type, Serialize)]
pub struct UnlockMsg {
    ethereum: ethereum::wallet::EthereumUnlock,
    bitcoin: bitcoin::wallet::BitcoinUnlock,
}

#[specta]
#[tauri::command]
pub async fn unlock_wallet(
    wallet_id: i32,
    passphrase: String,
    wallet_store: tauri::State<'_, WalletService>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
) -> Result<UnlockMsg, String> {
    let mnemonic = wallet_store.load(wallet_id, passphrase.clone())?;
    let eth_unlock_data =
        ethereum::wallet::unlock(&mnemonic, &passphrase).map_err(|e| e.to_string())?;
    let bitcoin_unlock_data =
        bitcoin::wallet::unlock(&mnemonic, &passphrase).map_err(|e| e.to_string())?;
    session_store.lock().await.start(session::Session::new(
        wallet_id,
        passphrase,
        Config::session_exp_duration(),
    ));
    Ok(UnlockMsg {
        ethereum: eth_unlock_data,
        bitcoin: bitcoin_unlock_data,
    })
}

#[specta]
#[tauri::command]
pub async fn forget_wallet(
    wallet_id: i32,
    repository: tauri::State<'_, WalletRepository>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
) -> Result<(), String> {
    session_store.lock().await.end();
    repository.delete(wallet_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[specta]
#[tauri::command]
pub async fn add_token(
    wallet_id: i32,
    chain: Chain,
    symbol: String,
    address: String,
    decimals: i32,
    db_pool: tauri::State<'_, crate::db::Pool>,
) -> Result<(), String> {
    let mut conn = db_pool
        .get()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let address_bytes = match chain {
        Chain::Ethereum => {
            // Parse Ethereum address (0x-prefixed hex string)
            let addr = alloy::primitives::Address::from_str(&address)
                .map_err(|e| format!("Invalid Ethereum address: {}", e))?;
            addr.as_slice().to_vec()
        }
        Chain::Bitcoin => {
            // For Bitcoin, addresses are typically base58 strings
            // Store as UTF-8 bytes for now (can be improved with proper base58 decoding later)
            address.as_bytes().to_vec()
        }
    };

    let new_token = Token {
        wallet_id,
        chain: i32::from(chain),
        symbol,
        address: address_bytes,
        decimals,
    };

    diesel::insert_into(schema::tokens::table)
        .values(&new_token)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert token: {}", e))?;

    Ok(())
}
