use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};

use crate::{
    chain_trait::{AssetTracker, ChainTrait, Persistable, SecureKey},
    eth::token::Token,
};

pub struct EthereumWallet {
    pub tracked_tokens: Vec<Token>,
}

pub struct Prk {
    signer: PrivateKeySigner,
}

impl SecureKey for Prk {
    type Material = PrivateKeySigner;
    fn expose(&self) -> &Self::Material {
        &self.signer
    }
}

impl ChainTrait for EthereumWallet {
    type Prk = Prk;
    type UnlockResult = EthereumUnlock;

    fn unlock(&self, prk: &Self::Prk) -> Result<Self::UnlockResult, String> {
        Ok(EthereumUnlock {
            address: prk.expose().address().to_string(),
        })
    }
}

impl Persistable for EthereumWallet {
    type Serialized = persistence::Wallet;

    fn serialize(&self) -> Result<Self::Serialized, String> {
        Ok(persistence::Wallet {
            tracked_tokens: self
                .tracked_tokens
                .iter()
                .map(|token| persistence::Token {
                    symbol: token.symbol.clone(),
                    address: token.address.to_string(),
                    decimals: token.decimals,
                })
                .collect(),
        })
    }

    fn deserialize(data: Self::Serialized) -> Result<Self, String> {
        let mut tracked_tokens = Vec::new();
        for token in data.tracked_tokens {
            let address = parse_addres(&token.address)?;
            tracked_tokens.push(Token {
                address,
                symbol: token.symbol,
                decimals: token.decimals,
            });
        }
        Ok(Self { tracked_tokens })
    }
}

impl AssetTracker<Token> for EthereumWallet {
    fn track(&mut self, asset: Token) -> Result<(), String> {
        // Check if token with same address is already tracked
        if self.tracked_tokens.iter().any(|t| *t == asset) {
            return Err(format!("Token {} already tracked", asset.address));
        }
        self.tracked_tokens.push(asset);
        Ok(())
    }

    fn untrack(&mut self, token: Token) -> Result<(), String> {
        let len_before = self.tracked_tokens.len();
        self.tracked_tokens.retain(|t| *t != token);
        if self.tracked_tokens.len() == len_before {
            return Err(format!("Token address '{}' not tracked", token.address));
        }
        Ok(())
    }

    fn list_tracked(&self) -> Vec<&Token> {
        self.tracked_tokens.iter().collect()
    }
}

impl EthereumWallet {
    pub fn get_tracked_token(&self, token: Address) -> Option<&Token> {
        self.tracked_tokens
            .iter()
            .find(|each| each.address == token)
    }
}

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}

pub fn build_prk(mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
    MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .password(passphrase)
        .derivation_path("m/44'/60'/0'/0")
        .unwrap()
        .index(0)
        .unwrap()
        .build()
        .map_err(|e| format!("fail to derive eth signer: {}", e))
        .map(|signer| Prk { signer })
}

#[derive(serde::Serialize, specta::Type)]
pub struct EthereumUnlock {
    pub address: String,
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Token {
        pub symbol: String,
        pub address: String,
        pub decimals: u8,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Wallet {
        pub tracked_tokens: Vec<Token>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mnemonic;

    #[test]
    fn test_construct_private_key() {
        let mnemonic = mnemonic::new().unwrap();
        let passphrase = "test passphrase";
        let result = build_prk(&mnemonic, passphrase);
        match result {
            Ok(prk) => println!("Success: {:?}", prk.signer.address()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
