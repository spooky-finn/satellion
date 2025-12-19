use alloy::primitives::{Address, address};
use once_cell::sync::Lazy;

use crate::{config::Chain, eth::token::Token, wallet};

pub static ETH: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
        "ETH".to_string(),
        18,
    )
});

/// Circle USD.
pub static USDC: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        "USDC".to_string(),
        6,
    )
});

/// Tether USD.
pub static USDT: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("dAC17F958D2ee523a2206206994597C13D831ec7"),
        "USDT".to_string(),
        6,
    )
});

pub fn get_default_tokens() -> Vec<wallet::Token> {
    [&USDC, &USDT]
        .iter()
        .map(|token| wallet::Token {
            address: token.address.to_string(),
            chain: Chain::Ethereum as u16,
            decimals: token.decimals,
            symbol: token.symbol.to_string(),
        })
        .collect()
}

pub const ETH_USD_PRICE_FEED: Address = address!("0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419");
