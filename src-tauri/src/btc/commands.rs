use serde::{Deserialize, Serialize};
use shush_rs::ExposeSecret;
use specta::{Type, specta};

use crate::{
    btc::{
        DerivedScript, Prk,
        address::{self, DerivePathSlice},
    },
    chain_trait::SecureKey,
    config::CONFIG,
    session::{SK, Session},
    wallet::Wallet,
};

fn build_prk(w: &Wallet) -> Result<Prk, String> {
    w.btc
        .build_prk(&w.mnemonic.expose_secret(), &w.passphrase.expose_secret())
}

#[specta]
#[tauri::command]
pub async fn btc_derive_address(
    label: String,
    index: u32,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session()?;
    let derive_path = address::DerivePath {
        change: address::Change::External,
        index,
        account: 0,
        network: CONFIG.bitcoin.network(),
        purpose: address::Purpose::Bip86,
    };
    if !wallet.btc.is_deriviation_path_free(derive_path.clone()) {
        return Err(format!("Deriviation index {} already occupied", index));
    }

    let prk = build_prk(wallet)?;
    let (_, address) =
        wallet
            .btc
            .derive_child(prk.expose(), CONFIG.bitcoin.network(), &derive_path)?;

    wallet.mutate_btc(|chain| {
        chain.add_child(label, derive_path.clone());
        let script = DerivedScript::new(address.script_pubkey(), derive_path);
        chain.add_script_of_interes(script);
        Ok(())
    })?;

    Ok(address.to_string())
}

#[specta]
#[tauri::command]
pub async fn btc_unoccupied_deriviation_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    Ok(sk
        .take_session()?
        .wallet
        .btc
        .unoccupied_deriviation_index(address::Change::External))
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
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddress>, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session()?;
    let prk = build_prk(wallet)?;
    Ok(wallet
        .btc
        .list_external_addresess()
        .map(|addr| {
            let (_, address) = wallet
                .btc
                .derive_child(
                    prk.expose(),
                    CONFIG.bitcoin.network(),
                    &addr.derive_path.clone(),
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

#[derive(specta::Type, Serialize, Deserialize)]

pub struct UtxoId {
    tx_id: String,
    vout: String,
}

#[derive(specta::Type, Serialize)]
pub struct Utxo {
    utxo_id: UtxoId,
    value: String,
    deriv_path: String,
    address_label: Option<String>,
}

const UTXO_DISPLAY_LIMIT: usize = 500;

#[specta]
#[tauri::command]
pub async fn btc_list_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<Utxo>, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session()?;

    let derivepath_label_map: std::collections::HashMap<DerivePathSlice, String> = wallet
        .btc
        .derived_addresses
        .iter()
        .map(|e| (e.derive_path.to_slice(), e.label.clone()))
        .collect();

    let mut utxos: Vec<_> = wallet
        .btc
        .utxos
        .values()
        .map(|utxo| {
            let label: Option<String> = match utxo.derive_path.change {
                address::Change::Internal => Some("Change".to_string()),
                address::Change::External => derivepath_label_map
                    .get(&utxo.derive_path.to_slice())
                    .cloned(),
            };
            Utxo {
                value: utxo.output.value.to_sat().to_string(),
                utxo_id: UtxoId {
                    tx_id: utxo.tx_id.to_string(),
                    vout: utxo.vout.to_string(),
                },
                deriv_path: utxo.derive_path.to_string(),
                address_label: label,
            }
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
