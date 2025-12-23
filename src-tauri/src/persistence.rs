use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    btc::{
        self,
        wallet::{AddressPurpose, persistence::BitcoinData},
    },
    config::{Config, constants::Chain},
    encryptor::{self, Envelope},
    eth::{self, wallet::persistence::EthereumData},
    wallet::Wallet,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistedWallet {
    pub name: String,
    pub mnemonic: String,
    pub bitcoin_data: BitcoinData,
    pub ethereum_data: EthereumData,
    pub last_used_chain: u16,
    pub created_at: u64,
    pub version: u8,
}

impl PersistedWallet {
    pub fn to_wallet(&self, passphrase: &str) -> Result<Wallet, String> {
        // Reconstruct Bitcoin Xpriv from mnemonic
        let bitcoin_xpriv = crate::btc::wallet::create_private_key(
            crate::config::CONFIG.bitcoin.network(),
            &self.mnemonic,
            passphrase,
        )?;

        // Reconstruct Ethereum signer from mnemonic
        let ethereum_signer = crate::eth::wallet::create_private_key(&self.mnemonic, passphrase)
            .map_err(|e| format!("Failed to create Ethereum private key: {}", e))?;

        Ok(Wallet {
            name: self.name.clone(),
            mnemonic: self.mnemonic.clone(),
            btc: crate::btc::wallet::WalletData {
                xpriv: bitcoin_xpriv,
                derived_addresses: self
                    .bitcoin_data
                    .childs
                    .iter()
                    .map(|addr| crate::btc::wallet::BitcoinAddress {
                        label: addr.label.clone(),
                        purpose: AddressPurpose::from(addr.purpose),
                        index: addr.index,
                    })
                    .collect(),
            },
            eth: crate::eth::wallet::WalletData {
                signer: ethereum_signer,
                tracked_tokens: self
                    .ethereum_data
                    .tracked_tokens
                    .iter()
                    .map(|t| {
                        let address = eth::wallet::parse_addres(&t.address).unwrap();
                        crate::eth::token::Token {
                            address,
                            symbol: t.symbol.clone(),
                            decimals: t.decimals,
                        }
                    })
                    .collect(),
            },
            last_used_chain: Chain::from(self.last_used_chain),
            created_at: self.created_at,
            version: self.version,
        })
    }

    pub fn from_wallet(wallet: &Wallet) -> Self {
        PersistedWallet {
            name: wallet.name.clone(),
            mnemonic: wallet.mnemonic.clone(),
            bitcoin_data: BitcoinData {
                childs: wallet
                    .btc
                    .derived_addresses
                    .iter()
                    .map(|addr| btc::wallet::persistence::ChildAddress {
                        label: addr.label.clone(),
                        purpose: addr.purpose.clone() as u8,
                        index: addr.index,
                    })
                    .collect(),
            },
            ethereum_data: EthereumData {
                tracked_tokens: wallet
                    .eth
                    .tracked_tokens
                    .iter()
                    .map(|t| crate::eth::wallet::persistence::Token {
                        symbol: t.symbol.clone(),
                        address: t.address.to_string(),
                        decimals: t.decimals,
                    })
                    .collect(),
            },
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

    pub fn load_as_wallet(&self, wallet_name: &str, passphrase: &str) -> Result<Wallet, String> {
        let data = FsRepository
            .get(wallet_name)
            .map_err(|e| format!("fail to load wallet from dist: {}", e))?;
        let decrypted_json = encryptor::decrypt(&data, passphrase.as_bytes())?;
        let persisted_wallet = serde_json::from_slice::<PersistedWallet>(&decrypted_json)
            .map_err(|e| format!("fail to parse json wallet into struct {}", e))?;
        persisted_wallet.to_wallet(passphrase)
    }

    pub fn store_wallet(&self, wallet: &Wallet, passphrase: &str) -> Result<(), String> {
        let persisted_wallet = PersistedWallet::from_wallet(wallet);
        let wallet_name = wallet.name.clone();
        let json = serde_json::to_string(&persisted_wallet)
            .map_err(|e| format!("fait to serialize persisted_wallet {}", e))?;
        let ciphertext = encryptor::encrypt(json.as_bytes(), passphrase.as_bytes())?;
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
    use crate::eth::{constants::USDT, wallet::persistence::Token};

    #[test]
    fn test_serialication() {
        let repository = Repository;
        let name = "Wallet 1".to_string();
        let passphrase = "1111";

        let persisted_wallet = PersistedWallet {
            name: name.clone(),
            mnemonic: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            created_at: 100,
            version: 0,
            last_used_chain: 1,
            bitcoin_data: BitcoinData {
                childs: vec![btc::wallet::persistence::ChildAddress {
                    purpose: 0,
                    index: 1,
                    label: "Secret contractor".to_string(),
                }],
            },
            ethereum_data: EthereumData {
                tracked_tokens: vec![Token {
                    address: USDT.address.to_string(),
                    decimals: 4,
                    symbol: "USDT".to_string()

                }],
            },
        };

        let wallet = persisted_wallet.to_wallet(passphrase).unwrap();
        repository.store_wallet(&wallet, passphrase).unwrap();

        let listed = repository.ls().unwrap();
        assert!(listed.contains(&FsRepository.sanitize_filename(&name)));

        let saved_wallet = repository.load_as_wallet(&name, passphrase).unwrap();
        assert_eq!(wallet, saved_wallet)
    }
}
