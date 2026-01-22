use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use shush_rs::{ExposeSecret, SecretBox};

use crate::{
    chain_trait::Persistable,
    config::{Config, constants::Chain},
    encryptor::{self, Envelope},
    wallet::Wallet,
    wallet_keeper::WalletKeeper,
};

#[derive(Serialize, Deserialize)]
pub struct SerializedWallet {
    pub name: String,
    pub mnemonic: String,
    pub bitcoin_data: crate::btc::persistence::Wallet,
    pub ethereum_data: crate::eth::persistence::Wallet,
    pub last_used_chain: u16,
    pub created_at: u64,
    pub version: u8,
}

impl SerializedWallet {
    pub fn to_model(&self, passphrase: SecretBox<String>) -> Result<Wallet, String> {
        Ok(Wallet {
            keeper: WalletKeeper::new(),
            name: self.name.clone(),
            mnemonic: SecretBox::new(Box::new(self.mnemonic.clone())),
            passphrase,
            // Use the Persistable trait for deserialization
            btc: crate::btc::BitcoinWallet::deserialize(self.bitcoin_data.clone())?,
            eth: crate::eth::EthereumWallet::deserialize(self.ethereum_data.clone())?,
            last_used_chain: Chain::from(self.last_used_chain),
            created_at: self.created_at,
            version: self.version,
        })
    }

    pub fn from_model(wallet: &Wallet) -> Self {
        SerializedWallet {
            name: wallet.name.clone(),
            mnemonic: wallet.mnemonic.expose_secret().to_string(),
            // Use the Persistable trait for serialization
            bitcoin_data: wallet
                .btc
                .serialize()
                .expect("Failed to serialize Bitcoin wallet data"),
            ethereum_data: wallet
                .eth
                .serialize()
                .expect("Failed to serialize Ethereum wallet data"),
            last_used_chain: u16::from(wallet.last_used_chain),
            created_at: wallet.created_at,
            version: wallet.version,
        }
    }
}

pub struct Repository;

impl Repository {
    pub fn ls(&self) -> Result<Vec<String>, io::Error> {
        FsRepository.ls()
    }

    pub fn load(&self, wallet_name: &str, passphrase: SecretBox<String>) -> Result<Wallet, String> {
        let data = FsRepository
            .get(wallet_name)
            .map_err(|e| format!("fail to load wallet from dist: {}", e))?;
        let decrypted_json = encryptor::decrypt(&data, passphrase.expose_secret().as_bytes())?;
        let persisted_wallet = serde_json::from_slice::<SerializedWallet>(&decrypted_json)
            .map_err(|e| format!("fail to parse json wallet into struct {}", e))?;
        persisted_wallet.to_model(passphrase)
    }

    pub fn store(&self, wallet: &Wallet) -> Result<(), String> {
        let persisted_wallet = SerializedWallet::from_model(wallet);
        let wallet_name = wallet.name.clone();
        let json = serde_json::to_string(&persisted_wallet)
            .map_err(|e| format!("fait to serialize persisted_wallet {}", e))?;
        let ciphertext = encryptor::encrypt(
            json.as_bytes(),
            wallet.passphrase.expose_secret().as_bytes(),
        )?;
        FsRepository
            .insert(&wallet_name, ciphertext)
            .map_err(|e| format!("fail to save wallet on disk {}", e))?;
        Ok(())
    }

    pub fn delete(&self, wallet_name: &str) -> Result<(), io::Error> {
        FsRepository.delete(wallet_name)
    }
}

struct FsRepository;

impl FsRepository {
    fn ls(&self) -> io::Result<Vec<String>> {
        let wallets_dir = Config::wallets_dir();
        let mut wallet_names = Vec::new();
        for entry in fs::read_dir(wallets_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
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
        let filename = format!("{}.json", self.sanitize_filename(wallet_name));
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
    use crate::{btc::address::make_hardened, eth::constants::USDT};

    #[test]
    fn test_serialication() {
        let repository = Repository;
        let name = "Wallet 1".to_string();
        let passphrase = SecretBox::new(Box::new("1111".to_string()));

        let persisted_wallet = SerializedWallet {
            name: name.clone(),
            mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            created_at: 100,
            version: 0,
            last_used_chain: 1,
            bitcoin_data: crate::btc::persistence::Wallet {
                cfilter_scanner_height: Some(0),
                childs: vec![crate::btc::persistence::ChildAddress {
                    label: "Secret contractor".to_string(),
                    devive_path: make_hardened([86,0,0,0,1])
                }],
                utxos: vec![],
            },
            ethereum_data: crate::eth::persistence::Wallet {
                tracked_tokens: vec![crate::eth::persistence::Token {
                    address: USDT.address.to_string(),
                    decimals: 4,
                    symbol: "USDT".to_string()

                }],
            },
        };

        let wallet = persisted_wallet.to_model(passphrase.clone()).unwrap();
        repository.store(&wallet).unwrap();

        let listed = repository.ls().unwrap();
        assert!(listed.contains(&FsRepository.sanitize_filename(&name)));

        let saved_wallet = repository
            .load(&name, passphrase)
            .expect("fail to load wallet");
        assert_eq!(
            wallet.mnemonic.expose_secret().to_string(),
            saved_wallet.mnemonic.expose_secret().to_string()
        )
    }
}
