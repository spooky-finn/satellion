use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};

use crate::{
    chain_wallet::{ChainWallet, Persistable, SecureKey, ZeroizableKey},
    eth::token::Token,
};

/// Ethereum-specific wallet data
pub struct WalletData {
    pub tracked_tokens: Vec<Token>,
}

pub struct Prk {
    pub signer: PrivateKeySigner,
}

impl SecureKey for Prk {
    type Material = PrivateKeySigner;

    fn expose(&self) -> &Self::Material {
        &self.signer
    }
}

// Ethereum's Prk implements ZeroizableKey because PrivateKeySigner handles zeroization internally
impl ZeroizableKey for Prk {}

impl WalletData {
    pub fn track_token(&mut self, token: Token) {
        self.tracked_tokens.push(token);
    }

    pub fn untrack_token(&mut self, address: &str) {
        if let Ok(addr) = parse_addres(address) {
            self.tracked_tokens.retain(|t| t.address != addr);
        }
    }
}

impl ChainWallet for WalletData {
    type Prk = Prk;
    type UnlockResult = EthereumUnlock;

    fn unlock(&self, prk: &Self::Prk) -> Result<Self::UnlockResult, String> {
        Ok(EthereumUnlock {
            address: prk.expose().address().to_string(),
        })
    }
}

impl Persistable for WalletData {
    type Serialized = persistence::EthereumData;

    fn serialize(&self) -> Result<Self::Serialized, String> {
        Ok(persistence::EthereumData {
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

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}

pub fn derive_prk(mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
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
    pub struct EthereumData {
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
        let result = derive_prk(&mnemonic, passphrase);
        match result {
            Ok(prk) => println!("Success: {:?}", prk.signer.address()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
