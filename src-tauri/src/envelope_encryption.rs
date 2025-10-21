//! Generic envelope encryption module for secure data storage.
//!
//! This module implements envelope encryption (key wrapping) for any sensitive data.
//! The encryption scheme uses:
//! - AES-256-GCM for authenticated encryption
//! - Argon2 for password-based key derivation
//! - Two-layer encryption: DEK (Data Encryption Key) encrypts plaintext,
//!   KEK (Key Encryption Key) derived from passphrase encrypts DEK
//!
//! # Storage Format
//! - `ciphertext`: [12 bytes dek_nonce][variable encrypted_data]
//! - `wrapped_key`: [12 bytes kek_nonce][48 bytes wrapped_dek]
//! - `kdf_salt`: 32 bytes for Argon2 KDF
use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, KeyInit};

pub const NONCE_SIZE: usize = 12;
pub const KEY_SIZE: usize = 32;
pub const SALT_SIZE: usize = 32;

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub wrapped_key: Vec<u8>,
    pub kdf_salt: Vec<u8>,
}

pub fn encrypt(plaintext: &[u8], passphrase: &[u8]) -> Result<EncryptedData, String> {
    let dek = rand::random::<[u8; KEY_SIZE]>();
    let dek_nonce = rand::random::<[u8; NONCE_SIZE]>();
    let kek_nonce = rand::random::<[u8; NONCE_SIZE]>();
    let kdf_salt = rand::random::<[u8; SALT_SIZE]>();
    let kek = derive_kek_from_passphrase(passphrase, &kdf_salt)?;
    let data_ciphertext = aes_encrypt(&dek, &dek_nonce, plaintext)?;
    let wrapped_dek = aes_encrypt(&kek, &kek_nonce, &dek)?;

    // Format: [dek_nonce][data_ciphertext]
    let mut ciphertext = dek_nonce.to_vec();
    // Format: [kek_nonce][wrapped_dek]
    let mut wrapped_key = kek_nonce.to_vec();
    ciphertext.extend_from_slice(&data_ciphertext);
    wrapped_key.extend_from_slice(&wrapped_dek);

    Ok(EncryptedData {
        ciphertext,
        wrapped_key,
        kdf_salt: kdf_salt.to_vec(),
    })
}
pub fn decrypt(encrypted: &EncryptedData, passphrase: &[u8]) -> Result<Vec<u8>, String> {
    let kek = derive_kek_from_passphrase(passphrase, &encrypted.kdf_salt)?;
    if encrypted.wrapped_key.len() < NONCE_SIZE {
        return Err("Invalid wrapped_key format".to_string());
    }
    let (kek_nonce_slice, wrapped_dek) = encrypted.wrapped_key.split_at(NONCE_SIZE);
    let kek_nonce: [u8; NONCE_SIZE] = kek_nonce_slice
        .try_into()
        .map_err(|_| "Invalid KEK nonce")?;
    let dek_bytes = aes_decrypt(&kek, &kek_nonce, wrapped_dek)
        .map_err(|_| "Invalid passphrase or corrupted wrapped key")?;
    let dek: [u8; KEY_SIZE] = dek_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Invalid DEK size")?;
    if encrypted.ciphertext.len() < NONCE_SIZE {
        return Err("Invalid ciphertext format".to_string());
    }
    let (dek_nonce_slice, data_ciphertext) = encrypted.ciphertext.split_at(NONCE_SIZE);
    let dek_nonce: [u8; NONCE_SIZE] = dek_nonce_slice
        .try_into()
        .map_err(|_| "Invalid DEK nonce")?;
    let plaintext = aes_decrypt(&dek, &dek_nonce, data_ciphertext)?;
    Ok(plaintext)
}

fn derive_kek_from_passphrase(passphrase: &[u8], salt: &[u8]) -> Result<[u8; KEY_SIZE], String> {
    let mut kek = [0u8; KEY_SIZE];
    argon2::Argon2::default()
        .hash_password_into(passphrase, salt, &mut kek)
        .map_err(|e| format!("KDF failed: {:?}", e))?;
    Ok(kek)
}

fn aes_encrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(&(*key).into());
    cipher
        .encrypt(&(*nonce).into(), plaintext)
        .map_err(|e| format!("AES-GCM encryption failed: {:?}", e))
}

fn aes_decrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(&(*key).into());
    cipher
        .decrypt(&(*nonce).into(), ciphertext)
        .map_err(|_| "AES-GCM decryption or authentication failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encrypt_decrypt_bytes() {
        let plaintext = b"secret data";
        let passphrase = b"my_secure_passphrase";
        let encrypted = encrypt(plaintext, passphrase).unwrap();
        let decrypted = decrypt(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    #[test]
    fn test_wrong_passphrase() {
        let plaintext = b"secret data";
        let passphrase = b"correct_passphrase";
        let encrypted = encrypt(plaintext, passphrase).unwrap();
        let result = decrypt(&encrypted, b"wrong_passphrase");
        assert!(result.is_err());
    }
    #[test]
    fn test_tampered_ciphertext() {
        let plaintext = b"secret data";
        let passphrase = b"passphrase";
        let mut encrypted = encrypt(plaintext, passphrase).unwrap();
        encrypted.ciphertext[20] ^= 0xFF;
        let result = decrypt(&encrypted, passphrase);
        assert!(result.is_err());
    }
    #[test]
    fn test_large_data() {
        let plaintext = vec![42u8; 10000];
        let passphrase = b"passphrase";
        let encrypted = encrypt(&plaintext, passphrase).unwrap();
        let decrypted = decrypt(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
