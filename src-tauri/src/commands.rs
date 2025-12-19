use std::sync::Arc;
use zeroize::Zeroize;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::Serialize;
use specta::{Type, specta};

use crate::{
    app_state::AppState,
    btc,
    config::{CONFIG, Chain, Config, constants},
    db::BlockHeader,
    eth, mnemonic,
    repository::wallet_repository::WalletRepositoryImpl,
    schema,
    session::{self, ChainSession},
    wallet::WalletRepository,
    wallet_service::WalletService,
};

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
    Ok(mnemonic::new()?)
}

#[specta]
#[tauri::command]
pub async fn create_wallet(
    mut mnemonic: String,
    mut passphrase: String,
    name: String,
    wallet_service: tauri::State<'_, WalletService>,
) -> Result<bool, String> {
    if passphrase.len() < constants::MIN_PASSPHRASE_LEN {
        return Err(format!(
            "Passphrase must contain at least {} characters",
            constants::MIN_PASSPHRASE_LEN
        ));
    }
    wallet_service.create(&mnemonic, &passphrase, &name)?;

    mnemonic.zeroize();
    passphrase.zeroize();
    Ok(true)
}

#[specta]
#[tauri::command]
pub async fn list_wallets(
    repository: tauri::State<'_, WalletRepositoryImpl>,
) -> Result<Vec<String>, String> {
    let available_wallets = repository.list_available().map_err(|e| e.to_string())?;
    Ok(available_wallets)
}

#[derive(Type, Serialize)]
pub struct UnlockMsg {
    ethereum: eth::wallet::EthereumUnlock,
    bitcoin: btc::wallet::BitcoinUnlock,
    last_used_chain: Chain,
}

#[specta]
#[tauri::command]
pub async fn unlock_wallet(
    wallet_name: String,
    mut passphrase: String,
    wallet_service: tauri::State<'_, WalletService>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
    wallet_repository: tauri::State<'_, WalletRepositoryImpl>,
) -> Result<UnlockMsg, String> {
    let mnemonic = wallet_service.load(&wallet_name, passphrase.clone())?;
    let (eth_unlock_data, eth_session) =
        eth::wallet::unlock(&mnemonic, &passphrase).map_err(|e| e.to_string())?;
    let (btc_unlock_data, btc_session) =
        btc::wallet::unlock(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let wallet = wallet_repository
        .get(&wallet_name)
        .map_err(|e| e.to_string())?;
    let last_used_chain = Chain::from(wallet.last_used_chain as u16);

    let mut session = session::Session::new(wallet_name, Config::session_exp_duration());
    session.add_chain_data(Chain::Bitcoin, ChainSession::from(btc_session));
    session.add_chain_data(Chain::Ethereum, ChainSession::from(eth_session));
    session_store.lock().await.start(session);

    passphrase.zeroize();
    Ok(UnlockMsg {
        ethereum: eth_unlock_data,
        bitcoin: btc_unlock_data,
        last_used_chain,
    })
}

#[specta]
#[tauri::command]
pub async fn forget_wallet(
    wallet_name: String,
    repository: tauri::State<'_, WalletRepositoryImpl>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::session::Store>>,
) -> Result<(), String> {
    session_store.lock().await.end();
    repository.delete(&wallet_name).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Type, Serialize)]
pub struct UIConfig {
    eth_anvil: bool,
}

#[specta]
#[tauri::command]
pub async fn get_config() -> Result<UIConfig, String> {
    Ok(UIConfig {
        eth_anvil: CONFIG.ethereum.anvil,
    })
}
