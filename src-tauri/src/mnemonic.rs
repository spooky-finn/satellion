use aes_gcm::aead::OsRng;
use alloy_signer_local::coins_bip39::{self, English};

pub fn new() -> Result<String, String> {
    let mut rng = OsRng;
    let mnemnonic = coins_bip39::Mnemonic::<English>::new_with_count(&mut rng, 12)
        .map_err(|e| format!("failed to generate mnemonic {e}"))?;
    Ok(mnemnonic.to_phrase())
}

pub fn validate(mnemonic: &str) -> Result<bool, String> {
    let mnemonic = coins_bip39::Mnemonic::<English>::new_from_phrase(mnemonic);
    match mnemonic {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Invalid mnemonic: {e}")),
    }
}

pub static TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
