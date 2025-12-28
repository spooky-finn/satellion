use std::str::FromStr;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::{Address, utils::format_units},
    providers::Provider,
};
use alloy_provider::{DynProvider, ext::AnvilApi};
use serde::{Deserialize, Serialize};
use shush_rs::ExposeSecret;
use specta::{Type, specta};

use crate::{
    config::Chain,
    eth::{
        self, PriceFeed,
        constants::{ETH, ETH_USD_PRICE_FEED},
        erc20_retriver::Erc20Retriever,
        fee_estimator::FeeMode,
        transfer_builder::TransferRequest,
        wallet::parse_addres,
    },
    session::AppSession,
    wallet_keeper::WalletKeeper,
};

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
    wallet_name: String,
    provider: tauri::State<'_, DynProvider>,
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
    price_feed: tauri::State<'_, PriceFeed>,
    session_keeper: tauri::State<'_, AppSession>,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<Balance, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;

    let provider = provider.inner();
    let address = parse_addres(&address)?;
    let wei_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;

    let token_balances = erc20_retriever
        .balances(address, session.wallet.eth.tracked_tokens.clone())
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

    session.wallet.last_used_chain = Chain::Ethereum;
    wallet_keeper.save_wallet(session)?;

    Ok(Balance {
        wei: wei_balance.to_string(),
        eth_price,
        tokens: token_balances,
    })
}

#[derive(Type, Deserialize, Debug, PartialEq)]
pub struct PrepareTxReqReq {
    wallet_name: String,
    token_address: String,
    amount: String,
    recipient: String,
    fee_mode: FeeMode,
}

#[derive(Type, Serialize, Debug, PartialEq)]
pub struct PrepareTxReqRes {
    estimated_gas: String,
    max_fee_per_gas: String,
    fee_ceiling: String,
    fee_in_usd: f64,
}

#[specta]
#[tauri::command]
pub async fn eth_prepare_send_tx(
    req: PrepareTxReqReq,
    tx_builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
    session_keeper: tauri::State<'_, AppSession>,
    price_feed: tauri::State<'_, PriceFeed>,
) -> Result<PrepareTxReqRes, String> {
    let PrepareTxReqReq {
        wallet_name,
        amount,
        fee_mode,
        recipient,
        token_address,
    } = req;
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;
    let prk = eth::wallet::derive_prk(
        &session.wallet.mnemonic.expose_secret(),
        &session.passphrase.expose_secret(),
    )?;
    let sender = prk.signer.address();
    let recipient = parse_addres(&recipient)?;
    let token_address = parse_addres(&token_address)?;

    let token = if token_address != ETH.address {
        let tracked_tokens = &session.wallet.eth.tracked_tokens;
        if let Some(t) = tracked_tokens.iter().find(|t| t.address == token_address) {
            t.clone()
        } else {
            return Err(format!(
                "Token with address '{}' not found in tracked tokens.",
                token_address
            ));
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
            fee_mode,
        })
        .await
        .map_err(|e| e.to_string())?;

    let eth_price = price_feed.get_price(ETH_USD_PRICE_FEED).await?;

    let fee_in_eth = format_units(res.fee_ceiling, "ether").map_err(|e| e.to_string())?;
    let eth_price_f64: f64 = eth_price
        .parse()
        .map_err(|_| "Failed to parse ETH price".to_string())?;
    let fee_in_eth_f64: f64 = fee_in_eth
        .parse()
        .map_err(|_| "Failed to parse fee".to_string())?;
    let fee_in_usd = eth_price_f64 * fee_in_eth_f64;

    let fee_ceiling_u64 = res.fee_ceiling.saturating_to::<u64>() / 10u64.pow(9);
    Ok(PrepareTxReqRes {
        estimated_gas: res.estimated_gas.to_string(),
        max_fee_per_gas: res.estimator.max_fee_per_gas.to_string(),
        fee_ceiling: fee_ceiling_u64.to_string(),
        fee_in_usd,
    })
}

#[specta]
#[tauri::command]
pub async fn eth_sign_and_send_tx(
    wallet_name: String,
    builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<String, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;
    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    let prk = eth::wallet::derive_prk(
        &session.wallet.mnemonic.expose_secret(),
        &session.passphrase.expose_secret(),
    )?;
    let hash = builder.sign_and_send_tx(&prk.signer).await?;
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
    symbol: String,
    decimals: i32,
}

#[specta]
#[tauri::command]
pub async fn eth_track_token(
    wallet_name: String,
    address: String,
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<TokenType, String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;

    let address = parse_addres(&address)?;
    let token_info = erc20_retriever
        .token_info(address)
        .await
        .map_err(|e| format!("Failed to fetch token info: {}", e))?;

    session.wallet.eth.track_token(crate::eth::token::Token {
        address,
        symbol: token_info.symbol.clone(),
        decimals: token_info.decimals as u8,
    });
    wallet_keeper.save_wallet(session)?;

    Ok(TokenType {
        chain: Chain::Ethereum,
        symbol: token_info.symbol,
        decimals: token_info.decimals as i32,
    })
}

#[specta]
#[tauri::command]
pub async fn eth_untrack_token(
    wallet_name: String,
    token_address: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    session_keeper: tauri::State<'_, AppSession>,
) -> Result<(), String> {
    let mut session_keeper = session_keeper.lock().await;
    let session = session_keeper.get(&wallet_name)?;

    session.wallet.eth.untrack_token(&token_address);
    wallet_keeper.save_wallet(session)?;
    Ok(())
}

#[specta]
#[tauri::command]
pub async fn eth_anvil_set_initial_balances(
    address: String,
    provider: tauri::State<'_, DynProvider>,
) -> Result<String, String> {
    use crate::eth::constants::USDT;
    use alloy::primitives::utils::{parse_ether, parse_units};

    let p = provider.inner();
    let addr = parse_addres(&address)?;
    let token = USDT.clone();

    p.anvil_set_balance(addr, parse_ether("10").unwrap())
        .await
        .map_err(|e| format!("Failed to set ETH balance: {}", e))?;
    p.anvil_deal_erc20(
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
