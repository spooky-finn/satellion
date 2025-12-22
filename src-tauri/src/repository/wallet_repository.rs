use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Chain, Config},
    wallet::{Token, Wallet, WalletRepository},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub name: String,
    pub encrypted_key: Vec<u8>,
    pub key_wrapped: Vec<u8>,
    pub kdf_salt: Vec<u8>,
    pub version: u8,
    pub created_at: String,
    pub last_used_chain: u16,
    pub tokens: Vec<Token>,
}

#[derive(Clone, Debug)]
pub struct WalletRepositoryImpl;

// Conversion traits between wallet and wallet file
impl From<Wallet> for WalletFile {
    fn from(wallet: Wallet) -> Self {
        Self {
            name: wallet.name,
            encrypted_key: wallet.encrypted_key,
            key_wrapped: wallet.key_wrapped,
            kdf_salt: wallet.kdf_salt,
            version: wallet.version,
            created_at: wallet.created_at,
            last_used_chain: wallet.last_used_chain,
            tokens: wallet.tokens,
        }
    }
}

impl From<WalletFile> for Wallet {
    fn from(wallet_file: WalletFile) -> Self {
        Self {
            name: wallet_file.name,
            encrypted_key: wallet_file.encrypted_key,
            key_wrapped: wallet_file.key_wrapped,
            kdf_salt: wallet_file.kdf_salt,
            version: wallet_file.version,
            created_at: wallet_file.created_at,
            last_used_chain: wallet_file.last_used_chain,
            tokens: wallet_file.tokens,
        }
    }
}

impl WalletRepository for WalletRepositoryImpl {
    fn new() -> Self {
        Config::ensure_wallets_dir();
        Self {}
    }

    fn list_available(&self) -> io::Result<Vec<String>> {
        let wallets_dir = Config::wallets_dir();
        let dir_entries = fs::read_dir(wallets_dir)?;

        let mut wallets = Vec::new();
        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to read and parse the wallet file
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue, // Skip unreadable files
            };

            let wname = match serde_json::from_str::<WalletFile>(&content) {
                Ok(wallet_file) => wallet_file.name,
                Err(_) => {
                    // If parsing fails, use filename as fallback
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string()
                }
            };

            wallets.push(wname);
        }

        wallets.sort();
        Ok(wallets)
    }

    fn insert(&self, wallet: Wallet) -> io::Result<()> {
        let wallet_file = WalletFile::from(wallet.clone());
        let content = serde_json::to_string_pretty(&wallet_file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(self.get_file_path(&wallet.name), content)?;
        Ok(())
    }

    fn get(&self, wname: &str) -> io::Result<Wallet> {
        self.assert_exists(wname)?;
        let content = fs::read_to_string(self.get_file_path(wname))?;
        let wallet_file = serde_json::from_str::<WalletFile>(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Wallet::from(wallet_file))
    }

    fn delete(&self, wname: &str) -> io::Result<()> {
        self.assert_exists(wname)?;
        fs::remove_file(self.get_file_path(wname))?;
        Ok(())
    }

    fn set_last_used_chain(&self, wname: &str, chain: Chain) -> Result<(), String> {
        let mut wallet = self.get(wname).map_err(|e| e.to_string())?;
        wallet.last_used_chain = chain as u16;

        let wallet_file = WalletFile::from(wallet);
        let content = serde_json::to_string_pretty(&wallet_file)
            .map_err(|e| format!("Failed to serialize wallet: {}", e))?;

        fs::write(self.get_file_path(wname), content)
            .map_err(|e| format!("Failed to write wallet file: {}", e))?;
        Ok(())
    }

    fn assert_exists(&self, wname: &str) -> io::Result<bool> {
        let exists = self.get_file_path(wname).exists();
        if !exists {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Wallet with name '{}' not found", wname),
            ));
        }
        Ok(true)
    }

    fn add_token(&self, wallet_name: &str, token: Token) -> Result<(), String> {
        let mut wallet_file = self.get_wallet_file(wallet_name)?;

        // Check if token already exists
        if wallet_file
            .tokens
            .iter()
            .any(|t| t.symbol == token.symbol && t.chain == token.chain)
        {
            return Err(format!(
                "Token {} already exists for this wallet",
                token.symbol
            ));
        }

        wallet_file.tokens.push(token);
        self.save_wallet_file(&wallet_file)?;
        Ok(())
    }

    fn remove_token(&self, wallet_name: &str, chain: Chain, symbol: &str) -> Result<(), String> {
        let mut wallet_file = self.get_wallet_file(wallet_name)?;
        let initial_count = wallet_file.tokens.len();

        wallet_file
            .tokens
            .retain(|t| !(t.chain == u16::from(chain) && t.symbol == symbol));

        if wallet_file.tokens.len() == initial_count {
            return Err(format!("Token {} not found for this wallet", symbol));
        }

        self.save_wallet_file(&wallet_file)?;
        Ok(())
    }

    fn get_tokens(&self, wallet_name: &str, chain: Chain) -> Result<Vec<Token>, String> {
        let wallet_file = self.get_wallet_file(wallet_name)?;
        Ok(wallet_file
            .tokens
            .into_iter()
            .filter(|t| t.chain == u16::from(chain))
            .collect())
    }

    fn get_token(&self, wallet_name: &str, chain: Chain, symbol: &str) -> Result<Token, String> {
        let wallet_file = self.get_wallet_file(wallet_name)?;
        wallet_file
            .tokens
            .into_iter()
            .find(|t| t.chain == u16::from(chain) && t.symbol == symbol)
            .ok_or_else(|| format!("Token {} not found for this wallet", symbol))
    }
}

impl WalletRepositoryImpl {
    /// Get the file path for a wallet with the given name
    fn get_file_path(&self, wname: &str) -> PathBuf {
        let mut path = Config::wallets_dir();
        let filename = format!("{}.json", self.sanitize_filename(wname));
        path.push(filename);
        path
    }

    /// Sanitize wallet name for use in filename
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    /// Helper method to get wallet file directly
    fn get_wallet_file(&self, wallet_name: &str) -> Result<WalletFile, String> {
        let wallet_path = self.get_file_path(wallet_name);

        if !wallet_path.exists() {
            return Err(format!("Wallet with name '{}' not found", wallet_name));
        }

        let content = fs::read_to_string(&wallet_path)
            .map_err(|e| format!("Failed to read wallet file: {}", e))?;
        let wallet_file = serde_json::from_str::<WalletFile>(&content)
            .map_err(|e| format!("Failed to parse wallet file: {}", e))?;

        Ok(wallet_file)
    }

    /// Helper method to save wallet file
    fn save_wallet_file(&self, wallet_file: &WalletFile) -> Result<(), String> {
        let wallet_path = self.get_file_path(&wallet_file.name);

        let content = serde_json::to_string_pretty(wallet_file)
            .map_err(|e| format!("Failed to serialize wallet: {}", e))?;

        fs::write(&wallet_path, content)
            .map_err(|e| format!("Failed to write wallet file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Chain;
    use crate::eth::constants::USDT;
    use crate::wallet::{Token, WalletRepository};

    #[test]
    fn test_token_management() {
        let repo = super::WalletRepositoryImpl::new();
        let wallet_name = "test_wallet_with_tokens";

        // Create a test wallet file directly
        let wallet_file = WalletFile {
            name: wallet_name.to_string(),
            encrypted_key: vec![1, 2, 3],
            key_wrapped: vec![4, 5, 6],
            kdf_salt: vec![7, 8, 9],
            version: 1,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            last_used_chain: 1,
            tokens: vec![],
        };

        // Save the wallet file
        let wallet_path = WalletRepositoryImpl.get_file_path(&wallet_name);
        std::fs::write(&wallet_path, serde_json::to_string(&wallet_file).unwrap()).unwrap();

        // Test adding a token
        let token = Token {
            chain: 1, // Ethereum
            symbol: "TEST".to_string(),
            address: USDT.address.to_string(),
            decimals: 18,
        };

        let result = repo.add_token(wallet_name, token.clone()).unwrap();
        assert_eq!(result, ());

        // Test getting tokens
        let tokens = repo.get_tokens(wallet_name, Chain::Ethereum).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol, "TEST");

        // Test getting specific token
        let found_token = repo
            .get_token(wallet_name, Chain::Ethereum, "TEST")
            .unwrap();
        assert_eq!(found_token.symbol, "TEST");
        assert_eq!(found_token.decimals, 18);

        // Test removing token
        let result = repo
            .remove_token(wallet_name, Chain::Ethereum, "TEST")
            .unwrap();
        assert_eq!(result, ());

        // Verify token is gone
        let tokens = repo.get_tokens(wallet_name, Chain::Ethereum).unwrap();
        assert_eq!(tokens.len(), 0);

        // Cleanup
        std::fs::remove_file(&wallet_path).unwrap();
    }
}
