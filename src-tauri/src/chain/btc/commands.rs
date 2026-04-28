use std::str::FromStr;

use bitcoin::Address;
use serde::{Deserialize, Serialize};
use specta::{Type, specta};

use crate::{
    chain::btc::{
        account::UtxoSelectionMethod,
        key_derivation::{Change, Proposal},
        service::{self, UtxoDto},
        tx_builder::{BuildPsbtParams, build_psbt, sign_psbt},
    },
    chain_trait::SecureKey,
    session::SK,
};

#[specta]
#[tauri::command]
pub async fn account_info(sk: tauri::State<'_, SK>) -> Result<service::ActiveAccountDto, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let account = wallet.btc.active_account()?;
    service::get_account_info(account, &prk, wallet.config.btc.network())
}

#[specta]
#[tauri::command]
pub async fn derive_external_address(
    label: String,
    index: u32,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    let taproot_key_path =
        wallet
            .btc
            .new_deriviation_path(Proposal::Bip86, Change::External, index)?;
    let derivation_scheme = taproot_key_path.with_label(label.clone());
    let child = derivation_scheme
        .path
        .derive(prk.expose())
        .map_err(|e| e.to_string())?;

    wallet
        .btc
        .get_mut_active_account()?
        .add_address(derivation_scheme);

    wallet.persist()?;
    Ok(child.taproot_address.to_string())
}

#[specta]
#[tauri::command]
pub async fn unoccupied_deriviation_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    Ok(account.unoccupied_address(Change::External))
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddressDto {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[specta]
#[tauri::command]
pub async fn get_external_addresess(
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddressDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let account = wallet.btc.active_account()?;
    let external_addresses: Vec<_> = account.get_external_addresess().collect();

    external_addresses
        .into_iter()
        .map(|scheme| {
            let child = scheme
                .path
                .derive(prk.expose())
                .map_err(|e| e.to_string())?;
            Ok(DerivedAddressDto {
                path: scheme.path.to_string(),
                label: scheme.label.clone(),
                address: child.taproot_address.to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

#[specta]
#[tauri::command]
pub async fn get_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let address_label_map = account.derive_path_label_map();

    let mut utxos: Vec<_> = account
        .utxos
        .values()
        .map(|utxo| utxo.to_dto(&address_label_map))
        .collect();

    utxos.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    const UTXO_DISPLAY_LIMIT: usize = 500;
    Ok(utxos.into_iter().take(UTXO_DISPLAY_LIMIT).collect())
}

#[specta]
#[tauri::command]
pub async fn sync_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let address_path_map = wallet
        .btc
        .active_account()?
        .derive_address_path_map(&prk, wallet.config.btc.network());

    let received_utxos = wallet
        .btc
        .server
        .get_utxos(address_path_map.clone())
        .await
        .map_err(|e| e.to_string())?;

    let account = wallet.btc.get_mut_active_account()?;
    account.set_utxos(received_utxos);

    let address_label_map = account.derive_path_label_map();
    let mut result = account
        .utxos
        .values()
        .map(|utxo| utxo.to_dto(&address_label_map))
        .collect::<Vec<_>>();

    result.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    wallet.persist()?;
    Ok(result)
}

#[derive(Type, Deserialize)]
pub struct BuildTx {
    pub value: String,
    pub recipient: String,
    pub utxo_selection_method: UtxoSelectionMethod,
}

#[derive(Type, Serialize)]
pub struct BuildTxResult {}

#[specta]
#[tauri::command]
pub async fn build_tx(req: BuildTx, sk: tauri::State<'_, SK>) -> Result<BuildTxResult, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let prk = wallet.btc_prk()?;
    let xpriv = prk.expose();

    let recipient: Address = Address::from_str(&req.recipient)
        .map_err(|e| format!("invalid recipient address: {e}"))?
        .require_network(wallet.config.btc.network())
        .map_err(|e| format!("recipient address network mismatch: {e}"))?;

    let send_value_sat = req
        .value
        .parse::<u64>()
        .map_err(|e| format!("invalid value: {e}"))?;

    let miner_fee_vbytes = wallet.btc.server.estimate_fee(1).await.unwrap();
    let pending_tx = build_psbt(&BuildPsbtParams {
        send_value_sat,
        recipient,
        utxo_selection_method: req.utxo_selection_method,
        miner_fee_vbytes,
        config: wallet.config.btc.clone(),
        account,
        xpriv,
    })
    .map_err(|e| format!("failed to build PSBT: {e}"))?;
    wallet.btc.pending_tx = Some(pending_tx);

    Ok(BuildTxResult {})
}

#[derive(Type, Deserialize)]
pub struct SendTx {}

#[derive(Type, Serialize)]
pub struct BroadcastResult {
    tx_id: String,
}

#[specta]
#[tauri::command]
pub async fn broadcast_tx(
    _req: SendTx,
    sk: tauri::State<'_, SK>,
) -> Result<BroadcastResult, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    let tx_build_res = wallet.btc.pending_tx.take().expect("pending tx not found");
    let psbt = tx_build_res.psbt;
    let tx = sign_psbt(psbt, &prk)?;

    let tx_id = wallet
        .btc
        .server
        .broadcast_tx(&tx)
        .await
        .map_err(|e| format!("fail to broadcast tx: {}", e))?;

    Ok(BroadcastResult { tx_id })
}
