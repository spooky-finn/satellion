use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{
    LocalSignerError, MnemonicBuilder, PrivateKeySigner, coins_bip39::English,
};

use crate::eth::token::Token;

/// Ethereum-specific wallet data
#[derive(Debug, PartialEq)]
pub struct WalletData {
    pub signer: PrivateKeySigner,
    pub tracked_tokens: Vec<Token>,
}

impl WalletData {
    pub fn unlock(&self) -> EthereumUnlock {
        EthereumUnlock {
            address: self.signer.address().to_string(),
        }
    }

    pub fn track_token(&mut self, token: Token) {
        self.tracked_tokens.push(token);
    }

    pub fn untrack_token(&mut self, address: &str) {
        if let Ok(addr) = parse_addres(address) {
            self.tracked_tokens.retain(|t| t.address != addr);
        }
    }
}

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}

pub fn create_private_key(
    mnemonic: &str,
    passphrase: &str,
) -> Result<PrivateKeySigner, LocalSignerError> {
    MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .password(passphrase)
        .derivation_path("m/44'/60'/0'/0")
        .unwrap()
        .index(0)
        .unwrap()
        .build()
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
        let result = create_private_key(&mnemonic, passphrase);
        match result {
            Ok(signer) => println!("Success: {:?}", signer.address()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
