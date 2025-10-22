//! Wallet-specific secure storage using envelope encryption.
//!
//! This module provides wallet-specific wrappers around the generic
//! envelope encryption module for storing cryptocurrency mnemonics.

use crate::{db, envelope_encryption};
use chrono::Utc;

pub fn create_encrypted_wallet(
    mnemonic: String,
    passphrase: String,
    wallet_name: String,
) -> Result<db::Wallet, String> {
    let encrypted = envelope_encryption::encrypt(mnemonic.as_bytes(), passphrase.as_bytes())?;
    let wallet = db::Wallet {
        id: 0,
        name: Some(wallet_name),
        encrypted_key: encrypted.ciphertext,
        key_wrapped: encrypted.wrapped_key,
        kdf_salt: encrypted.kdf_salt,
        version: 1,
        created_at: Utc::now().to_string(),
    };
    Ok(wallet)
}
pub fn decrypt_wallet(wallet: &db::Wallet, passphrase: String) -> Result<String, String> {
    let encrypted = envelope_encryption::EncryptedData {
        ciphertext: wallet.encrypted_key.clone(),
        wrapped_key: wallet.key_wrapped.clone(),
        kdf_salt: wallet.kdf_salt.clone(),
    };
    let mnemonic_bytes = envelope_encryption::decrypt(&encrypted, passphrase.as_bytes())?;
    let mnemonic =
        String::from_utf8(mnemonic_bytes).map_err(|_| "Invalid UTF-8 in decrypted mnemonic")?;

    Ok(mnemonic)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wallet_encrypt_decrypt() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let passphrase = "my_secure_passphrase";
        let wallet_name = "Test Wallet".to_string();
        let encrypted_wallet =
            create_encrypted_wallet(mnemonic.to_string(), passphrase.to_string(), wallet_name)
                .unwrap();
        let decrypted = decrypt_wallet(&encrypted_wallet, passphrase.to_string()).unwrap();
        assert_eq!(decrypted, mnemonic);
    }
    #[test]
    fn test_wallet_wrong_passphrase() {
        let mnemonic = "test mnemonic";
        let passphrase = "correct";
        let wallet_name = "Test".to_string();
        let encrypted_wallet =
            create_encrypted_wallet(mnemonic.to_string(), passphrase.to_string(), wallet_name)
                .unwrap();
        let result = decrypt_wallet(&encrypted_wallet, "wrong".to_string());
        assert!(result.is_err());
    }
}
