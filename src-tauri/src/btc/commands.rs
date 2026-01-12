use serde::{Deserialize, Serialize};
use shush_rs::ExposeSecret;
use specta::{Type, specta};

use crate::{
    btc::{DerivedScript, Prk, address},
    chain_trait::SecureKey,
    config::{CONFIG, Chain},
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
    wallet_name: String,
    label: String,
    index: u32,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session(&wallet_name)?;
    let derive_path = address::DerivePath {
        change: address::Change::External,
        index,
        network: CONFIG.bitcoin.network(),
    };
    if !wallet
        .btc
        .is_deriviation_index_available(derive_path.clone())
    {
        return Err(format!("Deriviation index {} already occupied", index));
    }

    let prk = build_prk(wallet)?;
    let child =
        wallet
            .btc
            .derive_child(prk.expose(), CONFIG.bitcoin.network(), derive_path.clone())?;

    wallet.last_used_chain = Chain::Bitcoin;
    wallet.mutate_btc(|chain| {
        chain.add_child(label, derive_path.clone());
        chain.add_script_of_interes(DerivedScript {
            script: child.1.script_pubkey(),
            derive_path,
        });
        Ok(())
    })?;

    Ok(child.1.to_string())
}

#[specta]
#[tauri::command]
pub async fn btc_unoccupied_deriviation_index(
    wallet_name: String,
    sk: tauri::State<'_, SK>,
) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    Ok(sk
        .take_session(&wallet_name)?
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
    wallet_name: String,
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddress>, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session(&wallet_name)?;
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

#[derive(specta::Type, Serialize, Deserialize)]

pub struct UTxOID {
    tx_id: String,
    vout: String,
}

#[derive(specta::Type, Serialize)]
pub struct Utxo {
    utxoid: UTxOID,
    value: String,
}

#[specta]
#[tauri::command]
pub async fn btc_list_utxos(
    wallet_name: String,
    sk: tauri::State<'_, SK>,
) -> Result<Vec<Utxo>, String> {
    let mut sk = sk.lock().await;
    let Session { wallet, .. } = sk.take_session(&wallet_name)?;
    Ok(wallet
        .btc
        .utxos
        .iter()
        .map(|utxo| Utxo {
            value: utxo.output.value.to_sat().to_string(),
            utxoid: UTxOID {
                tx_id: utxo.tx_id.to_string(),
                vout: utxo.vout.to_string(),
            },
        })
        .collect())
}
