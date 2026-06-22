use std::str::FromStr;

use bitcoin::{Address, Txid};
use specta::specta;

use crate::{
    chain::btc::{
        discovery::WalletDiscoverer,
        dtos::{
            ActiveAccountView, BroadcastTxRequest, BroadcastTxResponse, BuildTxRequest,
            BuildTxResponse, BumpFeeRequest, BumpFeeResponse, DerivedAddress, DiscoveryReportView,
            UtxoView,
        },
        fee_bump::{BuildCpfpParams, build_cpfp_psbt},
        fee_estimator::estimate_fee_rate,
        key_derivation::{Change, Proposal},
        tx_builder::{BuildPsbtParams, build_psbt, sign_psbt},
    },
    chain_trait::SecureKey,
    config::BlockChain,
    repository::{BtcChainData, NewTx, TxDirection, TxRepository, TxStatus},
    session::SK,
    utils,
};

#[specta]
#[tauri::command]
#[tracing::instrument(name = "account_info", skip_all, err)]
pub async fn account_info(sk: tauri::State<'_, SK>) -> Result<ActiveAccountView, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;
    let account = wallet.btc.active_account()?;
    account.info(&prk, wallet.config.btc.network())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "derive_external_address", skip_all, err)]
pub async fn derive_external_address(
    label: String,
    index: u32,
    proposal: Proposal,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;

    let key_derivation_path = wallet
        .btc
        .new_deriviation_path(proposal, Change::External, index)?
        .with_label(label.clone());
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
    Ok(child_key.address_for(proposal).to_string())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "next_unused_index", skip_all, err)]
pub async fn next_unused_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    Ok(account.keychain.next_unused_index(Change::External))
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "get_external_addresess", skip_all, err)]
pub async fn get_external_addresess(
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddress>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;
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
                address: key.address_for(scheme.path.purpose).to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "get_utxos", skip_all, err)]
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

/// Automated wallet discovery. Walks the derivation tree across all
/// supported schemes and collects every used path plus its UTXOs. Idempotent
/// against the current wallet state and safe to call multiple times.
#[specta]
#[tauri::command]
#[tracing::instrument(name = "discover_wallet", skip_all, err)]
pub async fn discover_wallet(sk: tauri::State<'_, SK>) -> Result<DiscoveryReportView, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;
    let network = wallet.config.btc.network();

    let discovered = WalletDiscoverer::new(&wallet.btc.server, prk.expose(), network)
        .discover()
        .await?;

    let report = wallet.btc.apply_discovery(discovered);
    wallet.persist()?;

    Ok(DiscoveryReportView {
        accounts: report.accounts,
        paths_added: report.paths_added as u32,
        utxos_added: report.utxos_added as u32,
        total_value_sat: report.total_value_sat.to_string(),
    })
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "sync_utxos", skip_all, err)]
pub async fn sync_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoView>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;
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
#[tracing::instrument(name = "build_tx", skip_all, err)]
pub async fn build_tx(
    req: BuildTxRequest,
    sk: tauri::State<'_, SK>,
) -> Result<BuildTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let prk = wallet.btc.prk()?;
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
#[tracing::instrument(name = "bump_fee_cpfp", skip_all, err)]
pub async fn bump_fee_cpfp(
    req: BumpFeeRequest,
    sk: tauri::State<'_, SK>,
) -> Result<BumpFeeResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc.prk()?;
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
#[tracing::instrument(name = "broadcast_tx", skip_all, err)]
pub async fn broadcast_tx(
    _req: BroadcastTxRequest,
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<BroadcastTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let tx_build_res = wallet.btc.pending_tx.take().expect("pending tx not found");
    let fee = tx_build_res.fee;
    let recipient = tx_build_res.recipient.clone();
    let send_value_sat = tx_build_res.send_value_sat;
    let psbt = tx_build_res.psbt;

    let prk = wallet.btc.prk()?;
    let tx = sign_psbt(psbt, &prk)?;
    let vsize = tx.vsize() as u32;

    let tx_id = wallet
        .btc
        .server
        .broadcast_tx(&tx)
        .await
        .map_err(|e| format!("fail to broadcast tx: {}", e))?;

    let chain_data = BtcChainData {
        vsize: Some(vsize),
        rbf: true,
        parent_tx_id: None,
        change_value_sat: None,
    };
    let _ = tx_repository.insert(NewTx {
        tx_hash: tx_id.clone(),
        wallet_name: wallet.name.clone(),
        chain: BlockChain::Bitcoin,
        account_index: wallet.btc.active_account as i32,
        direction: TxDirection::Outgoing,
        status: TxStatus::Pending,
        from_address: None,
        to_address: recipient,
        amount: send_value_sat as i64,
        fee: Some(fee as i32),
        block_height: None,
        chain_data: serde_json::to_value(chain_data).unwrap_or_default(),
        created_at: utils::now() as i64,
    });

    {
        // save change key
        let account = wallet.btc.get_active_account_mut()?;
        account.keychain.push(tx_build_res.change_key_path);
        wallet.persist()?;
    }

    Ok(BroadcastTxResponse { tx_id })
}
