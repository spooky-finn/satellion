use serde::{Deserialize, Serialize};

use crate::{
    chain::eth::{
        token::Token,
        wallet::{Account, EthereumWallet, parse_addres},
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
    pub accounts: Vec<AccountStored>,
    pub active_account: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountStored {
    pub index: u32,
    pub name: String,
    pub tracked_tokens: Vec<TokenStored>,
}

impl From<&EthereumWallet> for WalletStored {
    fn from(w: &EthereumWallet) -> Self {
        WalletStored {
            accounts: w
                .accounts
                .iter()
                .map(|account| AccountStored {
                    index: account.index,
                    name: account.name.clone(),
                    tracked_tokens: account
                        .tracked_tokens
                        .iter()
                        .map(TokenStored::from)
                        .collect(),
                })
                .collect(),
            active_account: w.active_account,
        }
    }
}

impl EthereumWallet {
    pub fn from_dto(dto: WalletStored, config: Config) -> Result<Self, String> {
        let accounts = dto
            .accounts
            .into_iter()
            .map(|account| {
                Ok::<Account, String>(Account {
                    index: account.index,
                    name: account.name,
                    tracked_tokens: account
                        .tracked_tokens
                        .into_iter()
                        .map(Token::try_from)
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EthereumWallet {
            config,
            accounts,
            active_account: dto.active_account,
        })
    }
}
