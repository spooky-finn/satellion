use serde::{Deserialize, Serialize};
use shush_rs::ExposeSecret;
use specta::{Type, specta};

use crate::{
    btc::{
        self, Prk,
        wallet::{Change, DerivePath},
    },
    chain_trait::{AssetTracker, SecureKey},
    config::{CONFIG, Chain},
    session::{AppSession, Session},
    wallet_keeper::WalletKeeper,
};

fn build_prk(s: &Session) -> Result<Prk, String> {
    btc::wallet::build_prk(
        &s.wallet.mnemonic.expose_secret(),
        &s.passphrase.expose_secret(),
    )
}

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
    let derive_path = DerivePath {
        change: Change::External,
        index,
    };
    if !session
        .wallet
        .btc
        .is_deriviation_index_available(derive_path.clone())
    {
        return Err(format!("Deriviation index {} already occupied", index));
    }

    let prk = build_prk(session)?;
    let child = session.wallet.btc.derive_child(
        prk.expose(),
        CONFIG.bitcoin.network(),
        derive_path.clone(),
    )?;

    session.wallet.last_used_chain = Chain::Bitcoin;
    session.wallet.btc.add_child(label, derive_path);

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
        .unoccupied_deriviation_index(Change::External))
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddress {
    pub label: String,
    pub address: String,
    pub deriv_path: String,
}

#[specta]
#[tauri::command]
pub async fn btc_list_derived_addresess(
    wallet_name: String,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<Vec<DerivedAddress>, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;
    let prk = build_prk(session)?;
    Ok(session
        .wallet
        .btc
        .list_tracked()
        .iter()
        .filter(|addr| addr.derive_path.change == Change::External)
        .map(|addr| {
            let (_, address) = session
                .wallet
                .btc
                .derive_child(
                    prk.expose(),
                    CONFIG.bitcoin.network(),
                    addr.derive_path.clone(),
                )
                .unwrap();
            DerivedAddress {
                deriv_path: addr.derive_path.to_string(),
                label: addr.label.clone(),
                address: address.to_string(),
            }
        })
        .collect())
}
