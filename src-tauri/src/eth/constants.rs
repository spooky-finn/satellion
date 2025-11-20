use crate::eth::token::Token;
use alloy::primitives::{Address, address};
use once_cell::sync::Lazy;

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

pub static DEFAULT_TOKENS: Lazy<Vec<Token>> =
    Lazy::<Vec<Token>>::new(|| vec![USDC.clone(), USDT.clone()]);

pub const ETH_USD_PRICE_FEED: Address = address!("0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419");
