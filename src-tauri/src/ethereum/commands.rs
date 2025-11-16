use crate::config::Chain;
use crate::ethereum::token::Token;
use crate::ethereum::token_manager::TokenManager;
use crate::repository::TokenRepository;
use crate::wallet_service::WalletService;
use crate::{db, ethereum, session};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::Address;
use alloy::primitives::utils::format_units;
use alloy::providers::Provider;
use alloy_provider::DynProvider;
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
pub async fn eth_chain_info(provider: tauri::State<'_, DynProvider>) -> Result<ChainInfo, String> {
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
    symbol: String,
    balance: String,
    decimals: u8,
    address: String,
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
    address: String,
    wallet_id: i32,
    provider: tauri::State<'_, DynProvider>,
    token_manager: tauri::State<'_, TokenManager>,
) -> Result<Balance, String> {
    let provider = provider.inner();
    let address = Address::from_str(&address).map_err(|e| e.to_string())?;
    let wei_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;
    let db_tokens = token_manager
        .load_all(wallet_id, Chain::Ethereum)
        .map_err(|e| format!("Failed to load token list: {}", e))?;

    let tokens: Vec<Token> = db_tokens
        .into_iter()
        .map(|t| {
            Token::new(
                Address::from_slice(&t.address),
                t.symbol.clone(),
                t.decimals as u8,
            )
        })
        .collect();

    let token_balances = token_manager
        .get_balances(address, tokens)
        .await
        .map_err(|e| e.to_string())?;

    let mut token_balances: Vec<TokenBalance> = token_balances
        .iter()
        .map(|b| TokenBalance {
            balance: b.balance.to_plain_string(),
            symbol: b.token.symbol.clone(),
            decimals: b.token.decimals,
            address: b.token.address.to_string(),
        })
        .collect();
    let eth = ethereum::constants::mainnet::ETH.clone();

    let eth_balance = format_units(wei_balance, "ether").map_err(|e| e.to_string())?;
    token_balances.push(TokenBalance {
        balance: eth_balance,
        symbol: eth.symbol.clone(),
        decimals: eth.decimals,
        address: eth.address.to_string(),
    });
    let price_feeder = ethereum::price_feed::PriceFeeder::new()?;
    let eth_price = price_feeder.get_eth_price().await?.to_string();
    Ok(Balance {
        wei: wei_balance.to_string(),
        eth_price,
        tokens: token_balances,
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
    token_symbol: String,
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

#[derive(Type, Serialize)]
pub struct TokenType {
    chain: Chain,
    address: String,
    symbol: String,
    decimals: i32,
}

#[specta]
#[tauri::command]
pub async fn eth_track_token(
    wallet_id: i32,
    address: String,
    token_repository: tauri::State<'_, TokenRepository>,
    token_manager: tauri::State<'_, TokenManager>,
) -> Result<TokenType, String> {
    let token_address =
        Address::from_str(&address).map_err(|e| format!("Invalid Ethereum address: {}", e))?;

    let token_info = token_manager
        .get_token_info(token_address)
        .await
        .map_err(|e| format!("Failed to fetch token info: {}", e))?;

    let token_entity = db::Token {
        wallet_id,
        chain: i32::from(Chain::Ethereum),
        symbol: token_info.symbol.clone(),
        address: token_address.as_slice().to_vec(),
        decimals: token_info.decimals as i32,
    };

    token_repository
        .insert(token_entity)
        .map_err(|e| format!("Failed to insert token: {}", e))?;
    Ok(TokenType {
        address,
        chain: Chain::Ethereum,
        symbol: token_info.symbol,
        decimals: token_info.decimals as i32,
    })
}

#[specta]
#[tauri::command]
pub async fn eth_untrack_token(
    wallet_id: i32,
    address: String,
    token_repository: tauri::State<'_, TokenRepository>,
) -> Result<bool, String> {
    let token_address =
        Address::from_str(&address).map_err(|e| format!("Invalid Ethereum address: {}", e))?;

    let rows_affected = token_repository
        .remove(wallet_id, Chain::Ethereum, token_address.as_slice())
        .map_err(|e| format!("Failed to remove token: {}", e))?;

    Ok(rows_affected > 0)
}
