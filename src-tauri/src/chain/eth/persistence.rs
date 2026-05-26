use serde::{Deserialize, Serialize};

use crate::{
    chain::eth::{
        token::Token,
        wallet::{EthereumWallet, parse_addres},
    },
    config::Config,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenStored {
    pub symbol: String,
    pub address: String,
    pub decimals: u8,
}

impl From<&Token> for TokenStored {
    fn from(t: &Token) -> Self {
        TokenStored {
            symbol: t.symbol.clone(),
            address: t.address.to_string(),
            decimals: t.decimals,
        }
    }
}

impl TryFrom<TokenStored> for Token {
    type Error = String;

    fn try_from(s: TokenStored) -> Result<Self, Self::Error> {
        Ok(Token {
            address: parse_addres(&s.address)?,
            symbol: s.symbol,
            decimals: s.decimals,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletStored {
    pub tracked_tokens: Vec<TokenStored>,
}

impl From<&EthereumWallet> for WalletStored {
    fn from(w: &EthereumWallet) -> Self {
        WalletStored {
            tracked_tokens: w.tracked_tokens.iter().map(TokenStored::from).collect(),
        }
    }
}

impl EthereumWallet {
    pub fn from_dto(dto: WalletStored, config: Config) -> Result<Self, String> {
        let tracked_tokens = dto
            .tracked_tokens
            .into_iter()
            .map(Token::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EthereumWallet {
            config,
            tracked_tokens,
            active_account: 0,
        })
    }
}
