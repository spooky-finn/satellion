use std::sync::Arc;
use zeroize::Zeroize;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::Serialize;
use specta::{Type, specta};

use crate::{
    app_state::AppState,
    btc::{self, neutrino::NeutrinoStarter},
    config::{CONFIG, Chain, Config, constants},
    db::BlockHeader,
    eth, mnemonic, schema,
    session::{AppSession, Session},
    wallet_keeper::WalletKeeper,
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
    mnemonic::new()
}

#[specta]
#[tauri::command]
pub async fn create_wallet(
    mut mnemonic: String,
    mut passphrase: String,
    name: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<bool, String> {
    if passphrase.len() < constants::MIN_PASSPHRASE_LEN {
        return Err(format!(
            "Passphrase must contain at least {} characters",
            constants::MIN_PASSPHRASE_LEN
        ));
    }
    wallet_keeper.create(&mnemonic, &passphrase, &name)?;

    mnemonic.zeroize();
    passphrase.zeroize();
    Ok(true)
}

#[specta]
#[tauri::command]
pub async fn list_wallets(
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<Vec<String>, String> {
    wallet_keeper.ls().map_err(|e| e.to_string())
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
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    session_keeper: tauri::State<'_, AppSession>,
    neutrino_starter: tauri::State<'_, NeutrinoStarter>,
) -> Result<UnlockMsg, String> {
    let wallet = wallet_keeper.load(&wallet_name, &passphrase)?;

    let ethereum = wallet.eth.unlock();
    let bitcoin = wallet.btc.unlock()?;

    let scripts = wallet.btc.derive_scripts_of_interes()?;
    let last_used_chain = wallet.last_used_chain;

    let session = Session::new(wallet, passphrase.clone(), Config::session_exp_duration());
    session_keeper.lock().await.start(session);

    // Start Bitcoin sync in background without waiting
    let neutrino_starter_clone = (*neutrino_starter).clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = neutrino_starter_clone.sync(scripts).await {
            eprintln!("Failed to start Bitcoin sync: {}", e);
        }
    });
    passphrase.zeroize();
    Ok(UnlockMsg {
        ethereum,
        bitcoin,
        last_used_chain,
    })
}

#[specta]
#[tauri::command]
pub async fn forget_wallet(
    wallet_name: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<(), String> {
    session_keeper.lock().await.end();
    wallet_keeper
        .delete(&wallet_name)
        .map_err(|e| e.to_string())?;
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
