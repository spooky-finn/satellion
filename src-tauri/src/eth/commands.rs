use crate::config::Chain;
use crate::eth::PriceFeed;
use crate::eth::constants::ETH_USD_PRICE_FEED;
use crate::eth::erc20_retriver::Erc20Retriever;
use crate::eth::wallet::parse_addres;
use crate::eth::{constants::ETH, token::Token, transfer_builder::TransferRequest};
use crate::{db, eth, repository::TokenRepository, session, wallet_service::WalletService};
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::{Address, utils::format_units},
    providers::Provider,
};
use alloy_provider::DynProvider;
use alloy_provider::ext::AnvilApi;
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
    token_repository: tauri::State<'_, TokenRepository>,
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
    price_feed: tauri::State<'_, PriceFeed>,
) -> Result<Balance, String> {
    let provider = provider.inner();
    let address = parse_addres(&address)?;
    let wei_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;
    let db_tokens = token_repository
        .load(wallet_id, Chain::Ethereum)
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

    let token_balances = erc20_retriever
        .balances(address, tokens)
        .await
        .map_err(|e| e.to_string())?;

    let mut token_balances: Vec<TokenBalance> = token_balances
        .iter()
        .map(|b| TokenBalance {
            balance: b.token.get_balance(b.balance).to_string(),
            symbol: b.token.symbol.clone(),
            decimals: b.token.decimals,
            address: b.token.address.to_string(),
        })
        .collect();
    let eth = eth::constants::ETH.clone();

    let eth_balance = format_units(wei_balance, "ether").map_err(|e| e.to_string())?;
    token_balances.push(TokenBalance {
        balance: eth_balance,
        symbol: eth.symbol.clone(),
        decimals: eth.decimals,
        address: eth.address.to_string(),
    });
    let eth_price = price_feed.get_price(ETH_USD_PRICE_FEED).await?.to_string();
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
    tx_builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
    session_store: tauri::State<'_, tokio::sync::Mutex<session::Store>>,
    storage: tauri::State<'_, WalletService>,
    token_repository: tauri::State<'_, TokenRepository>,
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
        eth::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

    let sender = signer.address();
    let recipient = parse_addres(&recipient)?;

    let token = if token_symbol.to_uppercase() != "ETH" {
        match token_repository.get(wallet_id, Chain::Ethereum, token_symbol.clone()) {
            Ok(t) => Token::new(
                Address::from_slice(&t.address),
                t.symbol.clone(),
                t.decimals as u8,
            ),
            Err(e) => {
                return Err(format!(
                    "Failed to load token info for {}: {}",
                    token_symbol, e
                ));
            }
        }
    } else {
        ETH.clone()
    };

    let mut builder = tx_builder.try_lock().map_err(|e| e.to_string())?;
    let res = builder
        .create_transfer(TransferRequest {
            token,
            raw_amount: amount,
            sender,
            recipient,
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(PrepareTxReqRes {
        estimated_gas: res.estimated_gas.to_string(),
        max_fee_per_gas: res.estimator.max_fee_per_gas.to_string(),
        cost: res.cost,
    })
}

#[specta]
#[tauri::command]
pub async fn eth_sign_and_send_tx(
    wallet_id: i32,
    builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
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
        eth::wallet::create_private_key(&mnemonic, &passphrase).map_err(|e| e.to_string())?;

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
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
) -> Result<TokenType, String> {
    let token_address = parse_addres(&address)?;
    let token_info = erc20_retriever
        .token_info(token_address)
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
    let token_address = parse_addres(&address)?;
    let rows_affected = token_repository
        .remove(wallet_id, Chain::Ethereum, token_address.as_slice())
        .map_err(|e| format!("Failed to remove token: {}", e))?;

    Ok(rows_affected > 0)
}

#[specta]
#[tauri::command]
pub async fn eth_anvil_set_initial_balances(
    address: String,
    provider: tauri::State<'_, DynProvider>,
) -> Result<String, String> {
    use crate::eth::constants::USDT;
    use alloy::primitives::utils::{parse_ether, parse_units};

    let provider = provider.inner();
    let addr = parse_addres(&address)?;

    provider
        .anvil_set_balance(addr, parse_ether("10").unwrap())
        .await
        .map_err(|e| format!("Failed to set ETH balance: {}", e))?;

    let token = USDT.clone();
    provider
        .anvil_deal_erc20(
            addr,
            token.address,
            parse_units("9999999", token.decimals)
                .unwrap()
                .get_absolute(),
        )
        .await
        .map_err(|e| format!("Failed to set USDT balance: {}", e))?;

    Ok("Initial balances set successfully: 10 ETH and 9,999,999 USDT".to_string())
}
