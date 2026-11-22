use std::str::FromStr;

use bitcoin::{Address, Transaction, Txid};
use specta::specta;

use crate::{
    chain::btc::{
        Prk,
        account::UtxoSelectionStrategy,
        discovery::WalletDiscoverer,
        dtos::{
            ActiveAccountView, BroadcastTxRequest, BroadcastTxResponse, BuildTxRequest,
            BuildTxResponse, BumpFeeRequest, BumpFeeResponse, ChainTip, DerivedAddress,
            DiscoveryReportView, UtxoView,
        },
        fee_bump::{BuildCpfpParams, build_cpfp_psbt},
        fee_estimator::estimate_fee_rate,
        key_derivation::{Change, Proposal},
        tx_builder::{BuildPsbtParams, BuildTxResult, build_psbt, sign_psbt},
    },
    chain_trait::SecureKey,
    config::BlockChain,
    repository::{
        BtcBroadcastScope, BtcChainData, NewTx, TxDirection, TxRecord, TxRepository, TxStatus,
    },
    session::SK,
    utils,
    wallet::Wallet,
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
#[tracing::instrument(name = "chain_tip", skip_all, err)]
pub async fn chain_tip(sk: tauri::State<'_, SK>) -> Result<ChainTip, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let (height, hash) = wallet.btc.server.network_status().await?;

    Ok(ChainTip {
        number: height.to_string(),
        hash,
    })
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
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<BuildTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    wallet.btc.pending_tx = None;
    let mut account = wallet.btc.active_account()?.clone();
    let reserved = tx_repository.reserved_btc_inputs(wallet.config.btc.network())?;
    if let UtxoSelectionStrategy::Manual(selected) = &req.utxo_selection_method {
        let mut seen = std::collections::HashSet::new();
        for item in selected {
            let outpoint: bitcoin::OutPoint = item.clone().try_into()?;
            if !seen.insert(outpoint) {
                return Err(format!("Input {outpoint} was selected more than once."));
            }
            if let Some(txid) = reserved.get(&outpoint) {
                return Err(format!(
                    "Input {outpoint} is reserved by transaction {txid}. Check that transaction before retrying."
                ));
            }
            if !account.utxo_set.entries.contains_key(&outpoint) {
                return Err(format!(
                    "Selected input {outpoint} is no longer available. Refresh the wallet and try again."
                ));
            }
        }
    }
    let had_inputs = !account.utxo_set.entries.is_empty();
    account
        .utxo_set
        .entries
        .retain(|outpoint, _| !reserved.contains_key(outpoint));
    if had_inputs && account.utxo_set.entries.is_empty() {
        return Err("No available Bitcoin inputs: one or more outputs are reserved by earlier transactions. Check transaction status before retrying.".to_string());
    }
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
        account: &account,
        xpriv,
    })
    .map_err(|e| format!("failed to build PSBT: {e}"))?;

    let response = BuildTxResponse {
        fee: pending_tx.fee,
        draft_tx_id: pending_tx.psbt.unsigned_tx.compute_txid().to_string(),
    };
    wallet.btc.pending_tx = Some(pending_tx);

    Ok(response)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "bump_fee_cpfp", skip_all, err)]
pub async fn bump_fee_cpfp(
    req: BumpFeeRequest,
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
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
    let fee = built.fee;
    let child_tx_id =
        broadcast_new(wallet, &tx_repository, &prk, built, Some(parent_tx_id)).await?;

    Ok(BumpFeeResponse {
        child_tx_id,
        child_fee: fee,
    })
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "broadcast_tx", skip_all, err)]
pub async fn broadcast_tx(
    req: BroadcastTxRequest,
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<BroadcastTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let draft = wallet
        .btc
        .pending_tx
        .as_ref()
        .ok_or("No reviewed transaction. Rebuild it before sending.")?;
    if draft.psbt.unsigned_tx.compute_txid().to_string() != req.draft_tx_id {
        return Err("The transaction draft changed. Review it again before sending.".into());
    }
    let draft = wallet
        .btc
        .pending_tx
        .take()
        .ok_or("No reviewed transaction")?;
    let prk = wallet.btc.prk()?;
    let tx_id = broadcast_new(wallet, &tx_repository, &prk, draft, None).await?;
    Ok(BroadcastTxResponse { tx_id })
}

async fn broadcast_new(
    wallet: &mut Wallet,
    repository: &TxRepository,
    prk: &Prk,
    draft: BuildTxResult,
    parent_tx_id: Option<Txid>,
) -> Result<String, String> {
    let tx = sign_psbt(draft.psbt, prk)?;
    let txid = tx.compute_txid().to_string();
    let account = wallet.btc.get_active_account_mut()?;
    for input in &tx.input {
        if !account
            .utxo_set
            .entries
            .contains_key(&input.previous_output)
        {
            return Err(format!(
                "Input {} is no longer available. Refresh the wallet and review the transaction again.",
                input.previous_output
            ));
        }
    }

    // Persist the change path before saving any transaction that could be retried.
    if let Some(change) = draft.change_key_path {
        if account.keychain.contains_path(change.path.clone()) {
            return Err(
                "The change address in this draft is already in use. Review a new draft.".into(),
            );
        }
        account.keychain.push(change);
        if let Err(error) = wallet.persist() {
            wallet.btc.get_active_account_mut()?.keychain.paths.pop();
            return Err(error);
        }
    }

    let key = prk.broadcast_key();
    let scope = broadcast_scope(wallet, &key);
    let chain_data = BtcChainData {
        vsize: Some(tx.vsize() as u32),
        rbf: tx.is_explicitly_rbf(),
        parent_tx_id: parent_tx_id.map(|txid| txid.to_string()),
        change_value_sat: draft.change_value_sat,
    };
    let record = NewTx {
        tx_hash: txid.clone(),
        wallet_name: scope.wallet_name.to_owned(),
        chain: BlockChain::Bitcoin,
        account_index: scope.account_index,
        direction: TxDirection::Outgoing,
        status: TxStatus::Unknown,
        from_address: None,
        to_address: draft.recipient,
        amount: draft.send_value_sat as i64,
        fee: Some(i32::try_from(draft.fee).map_err(|_| "Bitcoin fee exceeds the history limit")?),
        block_height: None,
        chain_data: serde_json::to_value(chain_data).map_err(|e| e.to_string())?,
        created_at: utils::now() as i64,
    };
    tracing::info!(%txid, amount_sat = record.amount, fee_sat = draft.fee, ?parent_tx_id, "Saving Bitcoin broadcast attempt");
    repository
        .record_btc_attempt(&scope, &tx, record)
        .map_err(|e| format!("Transaction {txid} was not broadcast: {e}"))?;
    broadcast_saved(wallet, repository, &scope, &tx).await
}

fn broadcast_scope<'a>(wallet: &'a Wallet, key: &'a str) -> BtcBroadcastScope<'a> {
    BtcBroadcastScope {
        wallet_name: &wallet.name,
        account_index: wallet.btc.active_account as i32,
        network: wallet.config.btc.network(),
        wallet_key: key,
    }
}

async fn broadcast_saved(
    wallet: &Wallet,
    repository: &TxRepository,
    scope: &BtcBroadcastScope<'_>,
    tx: &Transaction,
) -> Result<String, String> {
    let txid = tx.compute_txid().to_string();
    let inputs: Vec<_> = tx.input.iter().map(|input| input.previous_output).collect();
    tracing::info!(%txid, ?inputs, network = %scope.network, "Broadcasting saved Bitcoin transaction");
    let returned = wallet.btc.server.broadcast_tx(tx).await.map_err(|error| {
        format!("Broadcast outcome for transaction {txid} is unknown: {error}. Check its status or retry the saved transaction.")
    })?;
    if returned != txid {
        return Err(format!(
            "Broadcast outcome for transaction {txid} is unknown: response transaction ID did not match."
        ));
    }
    if let Err(error) = repository.acknowledge_btc_attempt(scope, &txid) {
        tracing::error!(%txid, %error, "Broadcast acknowledged but transaction status could not be updated");
    }
    tracing::info!(%txid, "Bitcoin broadcast acknowledged");
    Ok(txid)
}

/// Rebroadcast only the exact signed bytes saved for this wallet and account.
#[specta]
#[tauri::command]
#[tracing::instrument(name = "retry_btc_broadcast", skip_all, err)]
pub async fn retry_btc_broadcast(
    tx_id: String,
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<BroadcastTxResponse, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let key = wallet.btc.prk()?.broadcast_key();
    let scope = broadcast_scope(wallet, &key);
    let tx = tx_repository.btc_attempt_payload(&scope, &tx_id)?;
    let tx_id = broadcast_saved(wallet, &tx_repository, &scope, &tx).await?;
    Ok(BroadcastTxResponse { tx_id })
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "list_unresolved_btc_broadcasts", skip_all, err)]
pub async fn list_unresolved_btc_broadcasts(
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<Vec<TxRecord>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let key = wallet.btc.prk()?.broadcast_key();
    tx_repository.unresolved_btc_attempts(&broadcast_scope(wallet, &key))
}
