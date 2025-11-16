use crate::config::CONFIG;
use crate::ethereum::token_manager::TokenManager;
use crate::{db, ethereum::constants::mainnet::DEFAULT_TOKENS};
use alloy::network::Ethereum;
use alloy::providers::RootProvider;
use alloy_provider::{DynProvider, Provider, ProviderBuilder};
use std::time::Duration;
use tauri::Url;

pub fn new_provider() -> DynProvider {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    RootProvider::<Ethereum>::new_http(Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"))
        .erased()
}

pub fn new_provider_batched() -> DynProvider {
    let provider = new_provider();
    ProviderBuilder::new()
        .layer(alloy::providers::layers::CallBatchLayer::new().wait(Duration::from_millis(50)))
        .connect_provider(provider)
        .erased()
}

pub fn init_ethereum(token_manager: &TokenManager, wallet_id: i32) -> Result<usize, String> {
    let tokens: Vec<db::Token> = DEFAULT_TOKENS
        .iter()
        .map(|token| db::Token {
            wallet_id,
            chain: 1, // Ethereum
            symbol: token.symbol.to_string(),
            address: token.address.as_slice().to_vec(),
            decimals: token.decimals as i32,
        })
        .collect();

    token_manager
        .insert_default_tokens(tokens)
        .map_err(|e| format!("failed to insert default ethereum tokens {}", e))
}
