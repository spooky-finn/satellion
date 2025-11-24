use alloy_signer_local::coins_bip39::{self, English};
use bitcoin::secp256k1::rand::rngs::OsRng;

pub fn new() -> Result<String, String> {
    let mut rng = OsRng;
    let mnemnonic = coins_bip39::Mnemonic::<English>::new_with_count(&mut rng, 12)
        .map_err(|e| format!("failed to generate mnemonic {e}"))?;
    Ok(mnemnonic.to_phrase())
}

pub fn verify(mnemonic: String) -> Result<bool, String> {
    let mnemonic = coins_bip39::Mnemonic::<English>::new_from_phrase(&mnemonic);
    match mnemonic {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Invalid mnemonic {e}")),
    }
}
