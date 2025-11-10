use crate::ethereum::constants::token_symbol::TokenSymbol;
use crate::wallet_service::WalletService;
use crate::{ethereum, session};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, RootProvider};
use serde::Serialize;
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
    symbol: TokenSymbol,
    balance: String,
    decimals: u8,
    ui_precision: u8,
}

#[derive(Type, Serialize)]
pub struct Balance {
    wei: String,
    eth_price: String,
    tokens: Vec<TokenBalance>,
}

#[specta]
#[tauri::command]
pub async fn eth_get_balance(
    provider: tauri::State<'_, RootProvider>,
    address: String,
) -> Result<Balance, String> {
    let address = Address::from_str(&address).map_err(|e| e.to_string())?;
    let wei_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;
    let provider = provider.inner();
    let token_balances = ethereum::erc20::get_balances(provider, address)
        .await
        .map_err(|e| e.to_string())?;
    let mut tokens: Vec<TokenBalance> = token_balances
        .iter()
        .map(|b| TokenBalance {
            balance: b.balance.to_plain_string(),
            symbol: b.token.symbol,
            decimals: b.token.decimals,
            ui_precision: b.token.ui_precision,
        })
        .collect();
    let eth = ethereum::constants::mainnet::ETH.clone();

    let eth_balance = format_units(wei_balance, "ether").map_err(|e| e.to_string())?;
    tokens.push(TokenBalance {
        balance: eth_balance,
        symbol: eth.symbol.clone(),
        decimals: eth.decimals,
        ui_precision: eth.ui_precision,
    });
    let price_feeder = ethereum::price_feed::PriceFeeder::new()?;
    let eth_price = price_feeder.get_eth_price().await?.to_string();
    Ok(Balance {
        wei: wei_balance.to_string(),
        eth_price,
        tokens: tokens,
    })
}

#[derive(Type, Serialize, Debug, PartialEq)]
pub struct PrepareTxReqRes {
    estimated_gas: String,
    max_fee_per_gas: String,
    cost: String,
}

#[specta]
#[tauri::command]
pub async fn eth_prepare_send_tx(
    wallet_id: i32,
    token_symbol: TokenSymbol,
    amount: String,
    recipient: String,
    builder: tauri::State<'_, tokio::sync::Mutex<ethereum::TxBuilder>>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
    storage: tauri::State<'_, WalletService>,
) -> Result<PrepareTxReqRes, String> {
    let mut session_store = session_store.lock().await;
    let session = session_store.get(wallet_id);
    if session.is_none() {
        return Err("Session not found".to_string());
    }
    let passphrase = session.unwrap().passphrase.clone();
    let mnemonic = storage
        .load(wallet_id, passphrase.clone())
        .map_err(|e| e.to_string())?;
    let signer =
        ethereum::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let sender = signer.address();
    let recipient =
        Address::from_str(&recipient).map_err(|e| format!("Invalid recipient address: {e}"))?;

    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    let res = builder
        .eth_prepare_send_tx(token_symbol, amount, sender, recipient)
        .await?;
    Ok(PrepareTxReqRes {
        estimated_gas: res.estimated_gas.to_string(),
        max_fee_per_gas: res.max_fee_per_gas.to_string(),
        cost: res.cost,
    })
}

#[specta]
#[tauri::command]
pub async fn eth_sign_and_send_tx(
    wallet_id: i32,
    builder: tauri::State<'_, tokio::sync::Mutex<ethereum::TxBuilder>>,
    storage: tauri::State<'_, WalletService>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
) -> Result<String, String> {
    let mut session_store = session_store.lock().await;
    let session = session_store.get(wallet_id);
    if session.is_none() {
        return Err("Session not found".to_string());
    }
    let passphrase = session.unwrap().passphrase.clone();
    let mnemonic = storage
        .load(wallet_id, passphrase.clone())
        .map_err(|e| e.to_string())?;
    let signer =
        ethereum::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    let hash = builder.sign_and_send_tx(&signer).await?;
    Ok(hash.to_string())
}

#[specta]
#[tauri::command]
pub async fn eth_verify_address(address: String) -> Result<bool, String> {
    Address::from_str(&address).map_err(|e| e.to_string())?;
    Ok(true)
}
