use crate::ethereum::{constants::token_symbol::TokenSymbol, token::Token};
use alloy::primitives::address;
use once_cell::sync::Lazy;

pub static ETH: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
        TokenSymbol::ETH,
        18,
        6,
    )
});

/// Wrapped Ether.
pub static WETH: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        TokenSymbol::WETH,
        18,
        6,
    )
});

/// Wrapped Bitcoin.
pub static WBTC: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
        TokenSymbol::WBTC,
        8,
        6,
    )
});

/// Circle USD.
pub static USDC: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        TokenSymbol::USDC,
        6,
        0,
    )
});

/// Tether USD.
pub static USDT: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("dAC17F958D2ee523a2206206994597C13D831ec7"),
        TokenSymbol::USDT,
        6,
        0,
    )
});

/// Dai stablecoin.
pub static DAI: Lazy<Token> = Lazy::<Token>::new(|| {
    Token::new(
        address!("6B175474E89094C44Da98b954EedeAC495271d0F"),
        TokenSymbol::DAI,
        18,
        0,
    )
});

pub static TOKENS: Lazy<Vec<Token>> = Lazy::<Vec<Token>>::new(|| {
    vec![
        WETH.clone(),
        WBTC.clone(),
        USDC.clone(),
        USDT.clone(),
        DAI.clone(),
    ]
});

pub const ETH_USD_PRICE_FEED: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
