use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::utils::format_units,
    providers::Provider,
};
use alloy_provider::{DynProvider, ext::AnvilApi};
use serde::{Deserialize, Serialize};
use specta::{Type, specta};

use crate::{
    chain_trait::{AssetTracker, SecureKey},
    config::BlockChain,
    eth::{
        self, PriceFeed,
        constants::{ETH, ETH_USD_PRICE_FEED},
        erc20_retriver::Erc20Retriever,
        fee_estimator::FeeMode,
        transfer_builder::TransferPayload,
        wallet::parse_addres,
    },
    session::SK,
};

#[derive(Serialize, Type)]
pub struct NetworkStatus {
    block_number: String,
    block_hash: String,
    base_fee_per_gas: Option<String>,
}

#[specta]
#[tauri::command]
pub async fn get_network_status(
    provider: tauri::State<'_, DynProvider>,
) -> Result<NetworkStatus, String> {
    let block = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Latest))
        .await
        .map_err(|e| e.to_string())?;
    if block.is_none() {
        return Err("Block not found".to_string());
    }
    let block = block.unwrap();
    Ok(NetworkStatus {
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
pub struct WalletBalance {
    wei: String,
    tokens: Vec<TokenBalance>,
}

#[specta]
#[tauri::command]
pub async fn get_wallet_balance(
    address: String,
    provider: tauri::State<'_, DynProvider>,
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
    sk: tauri::State<'_, SK>,
) -> Result<WalletBalance, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let provider = provider.inner();
    let address = parse_addres(&address)?;
    let wei_balance = provider
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;

    let token_balances = erc20_retriever
        .balances(address, wallet.eth.tracked_tokens.clone())
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

    wallet.persist()?;
    Ok(WalletBalance {
        wei: wei_balance.to_string(),
        tokens: token_balances,
    })
}

#[derive(Type, Deserialize, Debug, PartialEq)]
pub struct TransferRequest {
    token_address: String,
    amount: String,
    recipient: String,
    fee_mode: FeeMode,
}

#[derive(Type, Serialize, Debug, PartialEq)]
pub struct TransferEstimation {
    estimated_gas: String,
    max_fee_per_gas: String,
    fee_ceiling: String,
    fee_in_usd: f64,
}

#[specta]
#[tauri::command]
pub async fn estimate_transfer(
    req: TransferRequest,
    tx_builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
    sk: tauri::State<'_, SK>,
    price_feed: tauri::State<'_, PriceFeed>,
) -> Result<TransferEstimation, String> {
    let TransferRequest {
        amount,
        fee_mode,
        recipient,
        token_address,
    } = req;
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.eth_prk()?;
    let sender = prk.expose().address();
    let recipient = parse_addres(&recipient)?;
    let token_address = parse_addres(&token_address)?;

    let token = if token_address != ETH.address {
        let tracked_tokens = &wallet.eth.tracked_tokens;
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
        .create_transfer(TransferPayload {
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
    Ok(TransferEstimation {
        estimated_gas: res.estimated_gas.to_string(),
        max_fee_per_gas: res.estimator.max_fee_per_gas.to_string(),
        fee_ceiling: fee_ceiling_u64.to_string(),
        fee_in_usd,
    })
}

#[specta]
#[tauri::command]
pub async fn execute_transfer(
    builder: tauri::State<'_, tokio::sync::Mutex<eth::TxBuilder>>,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let mut builder = builder.try_lock().map_err(|e| e.to_string())?;
    let prk = wallet.eth_prk()?;
    let hash = builder.sign_and_send_tx(prk.expose()).await?;
    Ok(hash.to_string())
}

#[derive(Type, Serialize)]
pub struct TrackedTokenInfo {
    chain: BlockChain,
    symbol: String,
    decimals: i32,
}

#[specta]
#[tauri::command]
pub async fn track_token(
    address: String,
    erc20_retriever: tauri::State<'_, Erc20Retriever>,
    sk: tauri::State<'_, SK>,
) -> Result<TrackedTokenInfo, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let address = parse_addres(&address)?;
    let token_info = erc20_retriever
        .token_info(address)
        .await
        .map_err(|e| format!("Failed to fetch token info: {}", e))?;

    wallet.mutate_eth(|eth| {
        eth.track(crate::eth::token::Token {
            address,
            symbol: token_info.symbol.clone(),
            decimals: token_info.decimals as u8,
        })?;
        Ok(())
    })?;

    Ok(TrackedTokenInfo {
        chain: BlockChain::Ethereum,
        symbol: token_info.symbol,
        decimals: token_info.decimals as i32,
    })
}

#[specta]
#[tauri::command]
pub async fn untrack_token(token_address: String, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let address = parse_addres(&token_address)?;
    let token = wallet
        .eth
        .get_tracked_token(address)
        .ok_or("Token with this address hasn't been tracked")?
        .clone();

    wallet.mutate_eth(|eth| {
        eth.untrack(token)?;
        Ok(())
    })?;
    Ok(())
}

#[specta]
#[tauri::command]
pub async fn anvil_set_initial_balances(
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
