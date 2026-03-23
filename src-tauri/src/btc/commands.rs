use serde::{Deserialize, Serialize};
use specta::{Type, specta};

use crate::{
    btc::{
        ActiveAccountDto,
        account::Account,
        key_derivation::{self, Change},
    },
    chain_trait::{AccountIndex, SecureKey},
    session::SK,
};

#[specta]
#[tauri::command]
pub async fn btc_switch_account(
    account: AccountIndex,
    sk: tauri::State<'_, SK>,
) -> Result<ActiveAccountDto, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    wallet.mutate_btc(|btc| {
        btc.switch_account(account);
        Ok(())
    })?;

    let prk = wallet.btc_prk()?;
    wallet.btc.active_account_info(&prk)
}

#[specta]
#[tauri::command]
pub async fn btc_derive_external_address(
    label: String,
    index: u32,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    let path = wallet.btc.new_deriviation_schema(Change::External, index)?;
    let address = wallet.mutate_btc(|btc| {
        let account = btc.get_mut_active_account()?;
        let (_, address) = account.add_child(&prk, label, path.clone())?;
        Ok(address)
    })?;

    Ok(address.to_string())
}

#[specta]
#[tauri::command]
pub async fn btc_unoccupied_deriviation_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let account = sk.wallet()?.btc.active_account()?;
    Ok(account.unoccupied_deriviation_index(key_derivation::Change::External))
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddressDto {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[specta]
#[tauri::command]
pub async fn btc_list_external_addresess(
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddressDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    let addresses = {
        let account = wallet.btc.active_account()?;
        account.get_external_addresess().collect::<Vec<_>>()
    };

    Ok(addresses
        .into_iter()
        .map(|addr| {
            let (_, address) = Account::derive_child(prk.expose(), &addr.path).unwrap();
            DerivedAddressDto {
                path: addr.path.to_string(),
                label: addr.label.clone(),
                address: address.to_string(),
            }
        })
        .collect())
}

#[derive(specta::Type, Serialize, Deserialize)]

pub struct UtxoId {
    tx_id: String,
    vout: String,
}

#[derive(specta::Type, Serialize)]
pub struct UtxoDto {
    utxo_id: UtxoId,
    value: String,
    deriv_path: String,
    address_label: Option<String>,
}

const UTXO_DISPLAY_LIMIT: usize = 500;

#[specta]
#[tauri::command]
pub async fn btc_list_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let schema_label_map = account.schema_label_map();

    let mut utxos: Vec<_> = account
        .utxos
        .values()
        .map(|utxo| UtxoDto {
            value: utxo.output.value.to_sat().to_string(),
            utxo_id: UtxoId {
                tx_id: utxo.tx_id.to_string(),
                vout: utxo.vout.to_string(),
            },
            deriv_path: utxo.derivation.to_string(),
            address_label: utxo.label(&schema_label_map),
        })
        .collect();

    utxos.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    Ok(utxos.into_iter().take(UTXO_DISPLAY_LIMIT).collect())
}
