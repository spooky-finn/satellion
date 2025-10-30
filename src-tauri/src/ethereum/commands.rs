use crate::ethereum;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use std::str::FromStr;

#[derive(serde::Serialize)]
pub struct ChainInfo {
    block_number: u64,
    block_hash: String,
    base_fee_per_gas: Option<u64>,
}

#[tauri::command]
pub async fn eth_chain_info(client: tauri::State<'_, RootProvider>) -> Result<ChainInfo, String> {
    let block = client
        .get_block(BlockId::Number(BlockNumberOrTag::Latest))
        .await
        .map_err(|e| e.to_string())?;
    if !block.is_some() {
        return Err("Block not found".to_string());
    }
    let block = block.unwrap();
    Ok(ChainInfo {
        block_number: block.header.number,
        block_hash: block.header.hash.to_string(),
        base_fee_per_gas: block.header.base_fee_per_gas,
    })
}

#[derive(serde::Serialize)]
pub struct TokenBalance {
    token_symbol: String,
    balance: String,
    decimals: u8,
    ui_precision: u8,
}

#[derive(serde::Serialize)]
pub struct Balance {
    wei: String,
    tokens: Vec<TokenBalance>,
}

#[tauri::command]
pub async fn eth_get_balance(
    client: tauri::State<'_, RootProvider>,
    address: String,
) -> Result<Balance, String> {
    let address = Address::from_str(&address).expect("Invalid Ethereum address");
    let eth_balance = client
        .get_balance(address)
        .await
        .map_err(|e| e.to_string())?;
    let provider = client.inner();
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
