use alloy_signer_local::{
    LocalSignerError, MnemonicBuilder, PrivateKeySigner, coins_bip39::English,
};

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
    address: String,
}

pub fn unlock(mnemonic: &str, passphrase: &str) -> Result<EthereumUnlock, LocalSignerError> {
    let signer = create_private_key(mnemonic, passphrase)?;
    Ok(EthereumUnlock {
        address: signer.address().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::mnemonic;

    use super::*;

    #[test]
    fn test_construct_private_key() {
        let mnemonic = mnemonic::new();
        let passphrase = "test passphrase";
        let result = create_private_key(&mnemonic, passphrase);
        match result {
            Ok(signer) => println!("Success: {:?}", signer.address()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
