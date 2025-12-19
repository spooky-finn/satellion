use std::io;

use serde::{Deserialize, Serialize};

use crate::config::Chain;

#[derive(Debug, PartialEq, Clone)]
pub struct Wallet {
    pub name: String,
    pub encrypted_key: Vec<u8>,
    pub key_wrapped: Vec<u8>,
    pub kdf_salt: Vec<u8>,
    pub version: u8,
    pub created_at: String,
    pub last_used_chain: u16,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token {
    pub chain: u16,
    pub symbol: String,
    pub address: String,
    pub decimals: u8,
}

pub trait WalletRepository: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;

    /// List all available wallets
    fn list_available(&self) -> io::Result<Vec<String>>;
    /// Insert a new wallet
    fn insert(&self, wallet: Wallet) -> io::Result<()>;
    /// Get a wallet by name
    fn get(&self, wname: &str) -> io::Result<Wallet>;
    /// Delete a wallet by name
    fn delete(&self, wname: &str) -> io::Result<()>;
    /// Asserts that wallet with the given name exists
    fn assert_exists(&self, wname: &str) -> io::Result<bool>;

    /// Set the last used chain for a wallet
    fn set_last_used_chain(&self, wname: &str, chain: Chain) -> Result<(), String>;
    /// Add a token to a wallet
    fn add_token(&self, wname: &str, token: Token) -> Result<(), String>;
    /// Remove a token from a wallet
    fn remove_token(&self, wname: &str, chain: Chain, symbol: &str) -> Result<(), String>;
    /// Get all tokens for a wallet
    fn get_tokens(&self, wname: &str, chain: Chain) -> Result<Vec<Token>, String>;
    /// Get a specific token for a wallet
    fn get_token(&self, wname: &str, chain: Chain, symbol: &str) -> Result<Token, String>;
}
