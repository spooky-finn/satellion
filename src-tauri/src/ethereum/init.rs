use crate::config::CONFIG;
use crate::ethereum::token_manager::TokenManager;
use crate::{db, ethereum::constants::mainnet::DEFAULT_TOKENS};
use alloy::network::Ethereum;
use alloy::providers::RootProvider;
use tauri::Url;

pub fn new_provider() -> RootProvider {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"),
    );
    provider
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
        .map_err(|e| format!("failed to insert default ethereum tokens ${e}"))
}
