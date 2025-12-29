use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};

use crate::{chain_wallet::ChainWallet, eth::token::Token};

/// Ethereum-specific wallet data
pub struct WalletData {
    pub tracked_tokens: Vec<Token>,
}

pub struct Prk {
    /// Signer will be zeroized on drop internally
    pub signer: PrivateKeySigner,
}

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
            address: prk.signer.address().to_string(),
        })
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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Token {
        pub symbol: String,
        pub address: String,
        pub decimals: u8,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
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
