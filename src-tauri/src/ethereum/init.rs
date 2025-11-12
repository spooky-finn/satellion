use crate::{db, ethereum::constants::mainnet::DEFAULT_TOKENS, token_tracker::TokenTracker};

pub fn init_ethereum(token_tracker: &TokenTracker, wallet_id: i32) -> Result<usize, String> {
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

    token_tracker
        .insert_default_tokens(tokens)
        .map_err(|e| format!("failed to insert default ethereum tokens ${e}"))
}
