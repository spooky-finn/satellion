use alloy_signer_local::{
    LocalSignerError, MnemonicBuilder, PrivateKeySigner, coins_bip39::English,
};

pub fn load_wallet(mnemonic: &str, passphrase: &str) -> Result<PrivateKeySigner, LocalSignerError> {
    let signer = MnemonicBuilder::<English>::default()
        .word_count(12)
        .password(passphrase)
        .phrase(mnemonic)
        .build()?;

    Ok(signer)
}
