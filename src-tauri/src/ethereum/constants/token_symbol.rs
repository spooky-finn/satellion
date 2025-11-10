use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Copy)]
pub enum TokenSymbol {
    ETH,
    WETH,
    WBTC,
    USDC,
    USDT,
    DAI,
}

impl TokenSymbol {
    pub fn as_str(&self) -> &str {
        match self {
            TokenSymbol::ETH => "ETH",
            TokenSymbol::WETH => "WETH",
            TokenSymbol::WBTC => "WBTC",
            TokenSymbol::USDC => "USDC",
            TokenSymbol::USDT => "USDT",
            TokenSymbol::DAI => "DAI",
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}
