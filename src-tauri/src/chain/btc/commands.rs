use std::str::FromStr;

use bitcoin::{Address, Txid};
use specta::specta;

use crate::{
    chain::btc::{
        dtos::{
            ActiveAccountView, BroadcastTxRequest, BroadcastTxResponse, BuildTxRequest,
            BuildTxResponse, BumpFeeRequest, BumpFeeResponse, DerivedAddress, UtxoView,
        },
        fee_bump::{BuildCpfpParams, build_cpfp_psbt},
        fee_estimator::estimate_fee_rate,
        key_derivation::{Change, Proposal},
        service::{self},
        tx_builder::{BuildPsbtParams, build_psbt, sign_psbt},
    },
    chain_trait::SecureKey,
    session::SK,
};

#[specta]
#[tauri::command]
pub async fn account_info(sk: tauri::State<'_, SK>) -> Result<ActiveAccountView, String> {
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
    let key_derivation_path = taproot_key_path.with_label(label.clone());
    let child_key = key_derivation_path
        .path
        .derive(prk.expose())
        .map_err(|e| e.to_string())?;

    wallet
        .btc
        .get_active_account_mut()?
        .keychain
        .push(key_derivation_path);

    wallet.persist()?;
    Ok(child_key.taproot_address.to_string())
}

#[specta]
#[tauri::command]
pub async fn next_unused_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    Ok(account.keychain.next_unused_index(Change::External))
}

#[specta]
#[tauri::command]
pub async fn get_external_addresess(
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddress>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let account = wallet.btc.active_account()?;
    let external_paths: Vec<_> = account
        .keychain
        .paths_by_change(&Change::External)
        .collect();

    external_paths
        .into_iter()
        .map(|scheme| {
            let key = scheme
                .path
                .derive(prk.expose())
                .map_err(|e| e.to_string())?;

            Ok(DerivedAddress {
                path: scheme.path.to_string(),
                label: scheme.label.clone(),
                address: key.taproot_address.to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

#[specta]
#[tauri::command]
pub async fn get_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoView>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let address_label_map = account.keychain.to_label_map();

    let mut utxos: Vec<_> = account
        .utxo_set
        .entries
        .values()
        .map(|utxo| utxo.to_view(&address_label_map))
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
pub async fn sync_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoView>, String> {
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

    let account = wallet.btc.get_active_account_mut()?;
    account.utxo_set.replace_all(received_utxos);

    let address_label_map = account.keychain.to_label_map();
    let mut result = account
        .utxo_set
        .entries
        .values()
        .map(|utxo| utxo.to_view(&address_label_map))
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

#[specta]
#[tauri::command]
pub async fn build_tx(
    req: BuildTxRequest,
    sk: tauri::State<'_, SK>,
) -> Result<BuildTxResponse, String> {
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

    let miner_fee_vbytes = estimate_fee_rate(&wallet.btc.server, &wallet.config.btc).await?;
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
    let change_key_path = pending_tx.change_key_path.clone();

    let fee = pending_tx.fee;
    wallet.btc.pending_tx = Some(pending_tx);
    wallet
        .btc
        .get_active_account_mut()?
        .keychain
        .push(change_key_path);
    wallet.persist()?;

    Ok(BuildTxResponse { fee })
}

#[specta]
#[tauri::command]
pub async fn bump_fee_cpfp(
    req: BumpFeeRequest,
    sk: tauri::State<'_, SK>,
) -> Result<BumpFeeResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let parent_tx_id =
        Txid::from_str(&req.parent_tx_id).map_err(|e| format!("invalid parent_tx_id: {e}"))?;

    let built = build_cpfp_psbt(&BuildCpfpParams {
        parent_tx_id,
        target_fee_rate_sat_vb: req.target_fee_rate_sat_vb,
        config: wallet.config.btc.clone(),
        account: wallet.btc.active_account()?,
        xpriv: prk.expose(),
    })?;
    let change_key_path = built.change_key_path.clone();
    let fee = built.fee;

    let tx = sign_psbt(built.psbt, &prk)?;
    let child_tx_id = wallet
        .btc
        .server
        .broadcast_tx(&tx)
        .await
        .map_err(|e| format!("fail to broadcast cpfp tx: {e}"))?;

    wallet
        .btc
        .get_active_account_mut()?
        .keychain
        .push(change_key_path);
    wallet.persist()?;

    Ok(BumpFeeResponse {
        child_tx_id,
        child_fee: fee,
    })
}

#[specta]
#[tauri::command]
pub async fn broadcast_tx(
    _req: BroadcastTxRequest,
    sk: tauri::State<'_, SK>,
) -> Result<BroadcastTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let tx_build_res = wallet.btc.pending_tx.take().expect("pending tx not found");
    let psbt = tx_build_res.psbt;

    let prk = wallet.btc_prk()?;
    let tx = sign_psbt(psbt, &prk)?;

    let tx_id = wallet
        .btc
        .server
        .broadcast_tx(&tx)
        .await
        .map_err(|e| format!("fail to broadcast tx: {}", e))?;

    {
        // save change key
        let account = wallet.btc.get_active_account_mut()?;
        account.keychain.push(tx_build_res.change_key_path);
        wallet.persist()?;
    }

    Ok(BroadcastTxResponse { tx_id })
}
