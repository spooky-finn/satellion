use crate::bitcoin;
use crate::bitcoin::wallet::AddressType;
use crate::repository::{AvailableWallet, Repository};
use crate::{app_state::AppState, db::BlockHeader, schema};
use crate::{ethereum, mnemonic, wallet_storage};
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
    Ok(mnemonic::new())
}

#[tauri::command]
pub async fn create_wallet(
    mnemonic: String,
    passphrase: String,
    name: String,
    repository: tauri::State<'_, Repository>,
) -> Result<bool, String> {
    mnemonic::verify(mnemonic.clone()).map_err(|e| e.to_string())?;
    let last_wallet_id = repository.last_wallet_id().map_err(|e| e.to_string())?;
    let mut wallet_name = name;
    if wallet_name.is_empty() {
        wallet_name = format!("Wallet {}", last_wallet_id + 1);
    }

    let wallet = wallet_storage::create_encrypted_wallet(
        last_wallet_id + 1,
        mnemonic,
        passphrase,
        wallet_name,
    )
    .map_err(|e| e.to_string())?;

    repository
        .insert_wallet(wallet)
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn get_available_wallets(
    repository: tauri::State<'_, Repository>,
) -> Result<Vec<AvailableWallet>, String> {
    let wallets_info = repository.available_wallets().map_err(|e| e.to_string())?;
    Ok(wallets_info)
}

#[derive(serde::Serialize)]
pub struct EthereumData {
    address: String,
}

#[derive(serde::Serialize)]
pub struct BitcoinData {
    address: String,
    change_address: String,
}

#[derive(serde::Serialize)]
pub struct UnlockMsg {
    wallet_id: i32,
    ethereum: EthereumData,
    bitcoin: BitcoinData,
}

#[tauri::command]
pub async fn unlock_wallet(
    wallet_id: i32,
    passphrase: String,
    repository: tauri::State<'_, Repository>,
) -> Result<UnlockMsg, String> {
    let wallet = repository
        .get_wallet_by_id(wallet_id)
        .map_err(|e| e.to_string())?;

    let mnemonic = wallet_storage::decrypt_wallet(&wallet, passphrase.clone());
    let mnemonic = match mnemonic {
        Ok(mnemonic) => mnemonic,
        Err(e) => return Err(e.to_string()),
    };

    let network = bitcoin::wallet::Network::Bitcoin;
    let eth_prk =
        ethereum::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let bitcoin_xprv = bitcoin::wallet::create_private_key(network, &mnemonic, &passphrase)
        .map_err(|e| e.to_string())?;

    let (_, bitcoin_main_receive_address) =
        bitcoin::wallet::derive_taproot_address(&bitcoin_xprv, network, AddressType::Receive, 0)
            .map_err(|e| e.to_string())?;
    let (_, bitcoin_main_change_address) =
        bitcoin::wallet::derive_taproot_address(&bitcoin_xprv, network, AddressType::Change, 0)
            .map_err(|e| e.to_string())?;

    let res = UnlockMsg {
        wallet_id,
        ethereum: EthereumData {
            address: eth_prk.address().to_string(),
        },
        bitcoin: BitcoinData {
            address: bitcoin_main_receive_address.to_string(),
            change_address: bitcoin_main_change_address.to_string(),
        },
    };
    Ok(res)
}

#[tauri::command]
pub async fn forget_wallet(
    wallet_id: i32,
    repository: tauri::State<'_, Repository>,
) -> Result<(), String> {
    repository
        .delete_wallet(wallet_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}
