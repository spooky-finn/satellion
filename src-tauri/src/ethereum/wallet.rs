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
        .build()?;
    Ok(signer)
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
