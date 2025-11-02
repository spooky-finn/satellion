use crate::config::CONFIG;
use alloy::network::Ethereum;
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::rpc::types::TransactionRequest;
use tauri::Url;

pub fn new() -> Result<RootProvider, String> {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"),
    );
    Ok(provider)
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct TxPresendInfo {
    pub gas_limit: u64,
    pub gas_price: u128,
}

pub async fn eth_prepare_send_tx(
    provider: &RootProvider,
    token_symbol: String,
    value: U256,
    recipient: Address,
) -> Result<TxPresendInfo, String> {
    if token_symbol != "ETH" {
        return Err("Only ETH is supported for now".to_string());
    }

    let tx = TransactionRequest::default()
        .with_to(recipient)
        .with_value(value);

    let estimated_gas = provider
        .estimate_gas(tx)
        .await
        .map_err(|e| format!("Failed to estimate gas: {}", e))?;

    let gas_price = provider
        .get_gas_price()
        .await
        .map_err(|e| format!("Failed to get gas price: {}", e))?;

    Ok(TxPresendInfo {
        gas_limit: estimated_gas,
        gas_price: gas_price,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_eth_prepare_send_tx() {
        let provider = new().unwrap();
        let token_symbol = "ETH".to_string();
        let value = U256::from(100);
        let recipient = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let res = eth_prepare_send_tx(&provider, token_symbol, value, recipient)
            .await
            .unwrap();
        println!("{:?}", res);
    }
}
