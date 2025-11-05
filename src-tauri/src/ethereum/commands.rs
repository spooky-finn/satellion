use crate::ethereum;
use crate::wallet_service::WalletService;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, RootProvider};
use serde::{Deserialize, Serialize};
use specta::{Type, specta};
use std::str::FromStr;

#[derive(Serialize, Type)]
pub struct ChainInfo {
    block_number: String,
    block_hash: String,
    base_fee_per_gas: Option<String>,
}

#[specta]
#[tauri::command]
pub async fn eth_chain_info(provider: tauri::State<'_, RootProvider>) -> Result<ChainInfo, String> {
    let block = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Latest))
        .await
        .map_err(|e| e.to_string())?;
    if block.is_none() {
        return Err("Block not found".to_string());
    }
    let block = block.unwrap();
    Ok(ChainInfo {
        block_number: block.header.number.to_string(),
        block_hash: block.header.hash.to_string(),
        base_fee_per_gas: block.header.base_fee_per_gas.map(|fee| fee.to_string()),
    })
}

#[derive(Type, Serialize)]
pub struct TokenBalance {
    token_symbol: String,
    balance: String,
    decimals: u8,
    ui_precision: u8,
}

#[derive(Type, Serialize)]
pub struct Balance {
    wei: String,
    tokens: Vec<TokenBalance>,
}

#[specta]
#[tauri::command]
pub async fn eth_get_balance(
    provider: tauri::State<'_, RootProvider>,
    address: String,
) -> Result<Balance, String> {
    let address = Address::from_str(&address).map_err(|e| e.to_string())?;
    let eth_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;
    let provider = provider.inner();
    let token_balances = ethereum::erc20::get_balances(provider, address)
        .await
        .map_err(|e| e.to_string())?;
    let tokens: Vec<TokenBalance> = token_balances
        .iter()
        .map(|balance| TokenBalance {
            balance: balance.balance.to_plain_string(),
            token_symbol: balance.token.symbol.clone(),
            decimals: balance.token.decimals,
            ui_precision: balance.token.ui_precision,
        })
        .collect();
    Ok(Balance {
        wei: eth_balance.to_string(),
        tokens: tokens,
    })
}

#[derive(Type, Deserialize)]
pub struct PrepareSendTxReq {
    token_symbol: String,
    amount: String,
    recipient: String,
    sender: String,
}

#[derive(Type, Serialize, Debug, PartialEq)]
pub struct PrepareTxReqRes {
    gas_limit: String,
    gas_price: String,
}

#[specta]
#[tauri::command]
pub async fn eth_prepare_send_tx(
    req: PrepareSendTxReq,
    builder: tauri::State<'_, tokio::sync::Mutex<ethereum::TxBuilder>>,
) -> Result<PrepareTxReqRes, String> {
    let token_symbol = req.token_symbol;
    let value = U256::from_str(&req.amount).map_err(|e| e.to_string())?;

    let recipient =
        Address::from_str(&req.recipient).map_err(|e| format!("Invalid recipient address: {e}"))?;
    let sender =
        Address::from_str(&req.sender).map_err(|e| format!("Invalid sender address: {e}"))?;

    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    let res = builder
        .eth_prepare_send_tx(token_symbol, value, sender, recipient)
        .await?;

    Ok(PrepareTxReqRes {
        gas_limit: res.gas_limit.to_string(),
        gas_price: res.gas_price.to_string(),
    })
}

#[specta]
#[tauri::command]
pub async fn eth_sign_and_send_tx(
    wallet_id: i32,
    passphrase: String,
    builder: tauri::State<'_, tokio::sync::Mutex<ethereum::TxBuilder>>,
    storage: tauri::State<'_, WalletService>,
) -> Result<(), String> {
    let mnemonic = storage
        .load(wallet_id, passphrase.clone())
        .map_err(|e| e.to_string())?;
    let signer =
        ethereum::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    builder.sign_and_send_tx(&signer).await?;
    Ok(())
}
