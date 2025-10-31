use alloy_signer_local::coins_bip39::{self, English};
use bitcoin::secp256k1::rand::rngs::OsRng;

pub fn new() -> String {
    let mut rng = OsRng;
    coins_bip39::Mnemonic::<English>::new_with_count(&mut rng, 12)
        .unwrap()
        .to_phrase()
}

pub fn verify(mnemonic: String) -> Result<bool, String> {
    let mnemonic = coins_bip39::Mnemonic::<English>::new_from_phrase(&mnemonic);
    match mnemonic {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Invalid mnemonic {e}")),
    }
}
