use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use shush_rs::{ExposeSecret, SecretBox};

use crate::{
    config::{Config, constants::BlockChain},
    encryptor::{self, Envelope},
    wallet::Wallet,
    wallet_keeper::WalletKeeper,
};

#[derive(Serialize, Deserialize)]
pub struct ChainSet {
    pub bitcoin: crate::btc::persistence::WalletStored,
    pub ethereum: crate::eth::persistence::WalletStored,
}

#[derive(Serialize, Deserialize)]
pub struct WalletEntity {
    pub name: String,
    pub mnemonic: String,
    pub chain_set: ChainSet,
    pub last_used_chain: u16,
    pub birth_date: Option<u64>,
    pub version: u16,
}

impl WalletEntity {
    pub fn to_model(&self, config: Config, passphrase: &str) -> Result<Wallet, String> {
        Ok(Wallet {
            keeper: WalletKeeper::default(),
            name: self.name.clone(),
            mnemonic: SecretBox::new(Box::new(self.mnemonic.clone())),
            passphrase: SecretBox::new(Box::new(passphrase.to_string())),
            btc: crate::chain::btc::BitcoinWallet::from_dto(
                self.chain_set.bitcoin.clone(),
                config.clone(),
            ),
            eth: crate::chain::eth::EthereumWallet::from_dto(
                self.chain_set.ethereum.clone(),
                config.clone(),
            )?,
            last_used_chain: BlockChain::from(self.last_used_chain),
            birth_date: self.birth_date,
            config,
            version: self.version,
        })
    }

    pub fn from_model(wallet: &Wallet) -> Self {
        WalletEntity {
            name: wallet.name.clone(),
            mnemonic: wallet.mnemonic.expose_secret().to_string(),
            chain_set: ChainSet {
                bitcoin: crate::chain::btc::persistence::WalletStored::from(&wallet.btc),
                ethereum: crate::chain::eth::persistence::WalletStored::from(&wallet.eth),
            },
            last_used_chain: u16::from(wallet.last_used_chain),
            birth_date: wallet.birth_date,
            version: wallet.version,
        }
    }
}

pub struct WalletRepository;

impl WalletRepository {
    pub fn ls(&self) -> Result<Vec<String>, io::Error> {
        FsRepository.ls()
    }

    pub fn load(
        &self,
        config: Config,
        wallet_name: &str,
        passphrase: &str,
    ) -> Result<Wallet, String> {
        let data = FsRepository
            .get(wallet_name)
            .map_err(|e| format!("fail to load wallet from dist: {}", e))?;
        let decrypted_json = encryptor::decrypt(&data, passphrase.as_bytes())?;
        let w = serde_json::from_slice::<WalletEntity>(&decrypted_json)
            .map_err(|e| format!("fail to parse json wallet into struct {}", e))?;
        w.to_model(config, passphrase)
    }

    pub fn save(&self, wallet: &Wallet) -> Result<(), String> {
        let persisted_wallet = WalletEntity::from_model(wallet);
        let wallet_name = wallet.name.clone();
        let json = serde_json::to_string(&persisted_wallet)
            .map_err(|e| format!("fait to serialize persisted_wallet {}", e))?;
        let ciphertext = encryptor::encrypt(
            json.as_bytes(),
            wallet.passphrase.expose_secret().as_bytes(),
            wallet.version,
        )?;
        FsRepository
            .insert(&wallet_name, ciphertext)
            .map_err(|e| format!("fail to save wallet on disk {}", e))?;
        Ok(())
    }

    pub fn rename(&self, wallet: &mut Wallet, new_name: &str) -> Result<(), String> {
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            return Err("Wallet name cannot be empty".to_string());
        }

        let old_name = wallet.name.clone();
        let old_sanitized = FsRepository.sanitize_filename(&old_name);
        let new_sanitized = FsRepository.sanitize_filename(&new_name);

        if old_sanitized != new_sanitized && FsRepository.get_file_path(&new_name).exists() {
            return Err(format!("A wallet named '{}' already exists", new_name));
        }

        wallet.name = new_name;
        self.save(wallet)?;

        if old_sanitized != new_sanitized {
            FsRepository
                .delete(&old_name)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn delete(&self, wallet_name: &str) -> Result<(), io::Error> {
        FsRepository.delete(wallet_name)
    }
}

struct FsRepository;

const EXTENSION: &str = "satellion";

impl FsRepository {
    fn ls(&self) -> io::Result<Vec<String>> {
        let wallets_dir = Config::wallets_dir();
        let mut wallet_names = Vec::new();
        for entry in fs::read_dir(wallets_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some(EXTENSION) {
                continue;
            }
            let wname = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            wallet_names.push(wname);
        }
        wallet_names.sort();
        Ok(wallet_names)
    }

    fn insert(&self, wallet_name: &str, data: Envelope) -> io::Result<()> {
        let content = serde_json::to_string(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(self.get_file_path(wallet_name), content)?;
        Ok(())
    }

    fn get(&self, wallet_name: &str) -> io::Result<Envelope> {
        self.assert_exists(wallet_name)?;
        let content = fs::read_to_string(self.get_file_path(wallet_name))?;
        let data = serde_json::from_str::<Envelope>(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    fn delete(&self, wallet_name: &str) -> io::Result<()> {
        self.assert_exists(wallet_name)?;
        fs::remove_file(self.get_file_path(wallet_name))?;
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

    /// Get the file path for a wallet with the given name
    fn get_file_path(&self, wallet_name: &str) -> PathBuf {
        let mut path = Config::wallets_dir();
        let filename = format!("{}.{EXTENSION}", self.sanitize_filename(wallet_name));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialication() {
        let repository = WalletRepository;
        let name = "Wallet 1".to_string();
        let passphrase = "1111";
        let config = Config::new();

        let btc = crate::chain::btc::BitcoinWallet::new(config.clone());
        let eth = crate::chain::eth::EthereumWallet::new(config.clone());

        let persisted_wallet = WalletEntity {
            name: name.clone(),
            mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            birth_date: None,
            last_used_chain: 1,
            chain_set: ChainSet {
                bitcoin: (&btc).into(),
                ethereum: (&eth).into(),
            },
            version: 1,
        };

        let wallet = persisted_wallet
            .to_model(config.clone(), passphrase)
            .unwrap();
        repository.save(&wallet).unwrap();

        let listed = repository.ls().unwrap();
        assert!(listed.contains(&FsRepository.sanitize_filename(&name)));

        let saved_wallet = repository
            .load(config.clone(), &name, &passphrase)
            .expect("fail to load wallet");
        assert_eq!(
            wallet.mnemonic.expose_secret().to_string(),
            saved_wallet.mnemonic.expose_secret().to_string()
        )
    }
}
