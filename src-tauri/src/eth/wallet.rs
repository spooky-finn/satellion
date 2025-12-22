use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{
    LocalSignerError, MnemonicBuilder, PrivateKeySigner, coins_bip39::English,
};

use crate::session::EthereumSession;

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}

pub fn create_private_key(
    mnemonic: &str,
    passphrase: &str,
) -> Result<PrivateKeySigner, LocalSignerError> {
    let signer = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .password(passphrase)
        .derivation_path("m/44'/60'/0'/0")
        .unwrap()
        .index(0)
        .unwrap()
        .build()?;
    Ok(signer)
}

#[derive(serde::Serialize, specta::Type)]
pub struct EthereumUnlock {
    pub address: String,
}

pub fn unlock(
    mnemonic: &str,
    passphrase: &str,
) -> Result<(EthereumUnlock, EthereumSession), LocalSignerError> {
    let signer = create_private_key(mnemonic, passphrase)?;
    Ok((
        EthereumUnlock {
            address: signer.address().to_string(),
        },
        EthereumSession { signer },
    ))
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
