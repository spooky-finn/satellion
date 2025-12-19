use std::sync::Arc;

use chrono::Utc;

use crate::{
    config::Chain,
    encryptor, mnemonic,
    wallet::{Wallet, WalletRepository},
};

pub struct WalletService {
    repository: Arc<dyn WalletRepository>,
}

impl WalletService {
    pub fn new(repository: Arc<dyn WalletRepository>) -> Self {
        Self {
            repository: repository.clone(),
        }
    }

    pub fn create(&self, mnemonic: &str, passphrase: &str, name: &str) -> Result<Wallet, String> {
        mnemonic::verify(mnemonic).map_err(|e| e.to_string())?;
        let default_name = self.generate_default_wallet_name()?;

        let mut name = name;
        if name.is_empty() {
            name = default_name.as_ref();
        }

        let envelope = encryptor::encrypt(mnemonic.as_bytes(), passphrase.as_bytes())?;
        let wallet = Wallet {
            name: name.to_string(),
            encrypted_key: envelope.ciphertext,
            key_wrapped: envelope.wrapped_key,
            kdf_salt: envelope.kdf_salt,
            version: 1,
            created_at: Utc::now().to_string(),
            last_used_chain: Chain::Bitcoin as u16,
            tokens: crate::eth::constants::get_default_tokens(),
        };
        self.repository
            .insert(wallet.clone())
            .map_err(|e| e.to_string())?;
        Ok(wallet)
    }

    pub fn load(&self, wallet_name: &str, passphrase: String) -> Result<String, String> {
        let wallet = self
            .repository
            .get(wallet_name)
            .map_err(|e| e.to_string())?;

        let envelope = encryptor::EncryptedData {
            ciphertext: wallet.encrypted_key.clone(),
            wrapped_key: wallet.key_wrapped.clone(),
            kdf_salt: wallet.kdf_salt.clone(),
        };
        let mnemonic_bytes = encryptor::decrypt(&envelope, passphrase.as_bytes())?;
        let mnemonic =
            String::from_utf8(mnemonic_bytes).map_err(|_| "Invalid UTF-8 in decrypted mnemonic")?;

        Ok(mnemonic)
    }

    /// Generate a default wallet name using the next available ordinal number
    fn generate_default_wallet_name(&self) -> Result<String, String> {
        let existing_wallets = self
            .repository
            .list_available()
            .map_err(|e| e.to_string())?;

        let mut max_ordinal = 0;
        for wallet_name in existing_wallets {
            if !wallet_name.starts_with("Wallet_") {
                continue;
            }

            if let Some(ordinal_str) = wallet_name.strip_prefix("Wallet_") {
                if let Ok(ordinal) = ordinal_str.parse::<u32>() {
                    max_ordinal = max_ordinal.max(ordinal);
                }
            }
        }

        let next_ordinal = max_ordinal + 1;
        Ok(format!("Wallet_{}", next_ordinal))
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::wallet_repository::WalletRepositoryImpl;

    use super::*;
    #[test]
    fn test_wallet_encrypt_decrypt() {
        let repository = Arc::new(WalletRepositoryImpl::new());
        let wallet_service = WalletService::new(repository.clone());

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let passphrase = "my_secure_passphrase";
        let wallet_name = wallet_service
            .generate_default_wallet_name()
            .expect("fail to generate wallet name");

        wallet_service
            .create(mnemonic, &passphrase, &wallet_name)
            .unwrap();
        let decrypted = wallet_service
            .load(&wallet_name, passphrase.to_string())
            .unwrap();
        assert_eq!(decrypted, mnemonic);
        repository.delete(&wallet_name).unwrap();
    }

    #[test]
    fn test_wallet_wrong_passphrase() {
        let repository = Arc::new(WalletRepositoryImpl::new());
        let wallet_service = WalletService::new(repository.clone());

        let mnemonic = "invalid mnemonic phrase";
        let passphrase = "my_secure_passphrase";
        let wallet_name = wallet_service
            .generate_default_wallet_name()
            .expect("fail to generate wallet name");

        let encrypted_wallet = wallet_service
            .create(mnemonic, passphrase, wallet_name.as_ref())
            .map_err(|e| e.to_string());
        assert!(encrypted_wallet.is_err()); // Invalid mnemonic should fail
    }
}
