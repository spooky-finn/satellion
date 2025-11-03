//! Wallet-specific secure storage using envelope encryption.
//!
//! This module provides wallet-specific wrappers around the generic
//! envelope encryption module for storing cryptocurrency mnemonics.

use crate::{db, envelope_encryption, mnemonic, repository::Repository};
use chrono::Utc;

pub struct WalletService {
    repository: Repository,
}

impl WalletService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub fn create(
        &self,
        mnemonic: String,
        passphrase: String,
        wallet_name: String,
    ) -> Result<db::Wallet, String> {
        mnemonic::verify(mnemonic.clone()).map_err(|e| e.to_string())?;
        let last_wallet_id = self
            .repository
            .last_wallet_id()
            .map_err(|e| e.to_string())?;

        let mut wallet_name = wallet_name;
        if wallet_name.is_empty() {
            wallet_name = format!("Wallet {}", last_wallet_id + 1);
        }
        let envelope = envelope_encryption::encrypt(mnemonic.as_bytes(), passphrase.as_bytes())?;
        let wallet = db::Wallet {
            id: last_wallet_id + 1,
            name: Some(wallet_name),
            encrypted_key: envelope.ciphertext,
            key_wrapped: envelope.wrapped_key,
            kdf_salt: envelope.kdf_salt,
            version: 1,
            created_at: Utc::now().to_string(),
        };
        self.repository
            .insert_wallet(wallet.clone())
            .map_err(|e| e.to_string())?;
        Ok(wallet)
    }

    pub fn load(&self, wallet_id: i32, passphrase: String) -> Result<String, String> {
        let wallet = self
            .repository
            .get_wallet_by_id(wallet_id)
            .map_err(|e| e.to_string())?;

        let envelope = envelope_encryption::EncryptedData {
            ciphertext: wallet.encrypted_key.clone(),
            wrapped_key: wallet.key_wrapped.clone(),
            kdf_salt: wallet.kdf_salt.clone(),
        };
        let mnemonic_bytes = envelope_encryption::decrypt(&envelope, passphrase.as_bytes())?;
        let mnemonic =
            String::from_utf8(mnemonic_bytes).map_err(|_| "Invalid UTF-8 in decrypted mnemonic")?;

        Ok(mnemonic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wallet_encrypt_decrypt() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let passphrase = "my_secure_passphrase";
        let wallet_name = "Test Wallet".to_string();

        let repository = Repository::new(db::connect());
        let storage = WalletService::new(repository);

        let encrypted_wallet = storage
            .create(mnemonic.to_string(), passphrase.to_string(), wallet_name)
            .unwrap();
        let decrypted = storage
            .load(encrypted_wallet.id, passphrase.to_string())
            .unwrap();
        assert_eq!(decrypted, mnemonic);
    }
    #[test]
    fn test_wallet_wrong_passphrase() {
        let mnemonic = "test mnemonic";
        let passphrase = "correct";
        let wallet_name = "Test".to_string();
        let repository = Repository::new(db::connect());
        let storage = WalletService::new(repository);
        let encrypted_wallet = storage
            .create(mnemonic.to_string(), passphrase.to_string(), wallet_name)
            .map_err(|e| e.to_string());
        assert!(encrypted_wallet.is_err());
    }
}
