use std::{collections::HashSet, fmt::Display, str::FromStr};

use bip39::Language;
use bip157::ScriptBuf;
pub use bitcoin::network::Network;
use bitcoin::{
    Address,
    bip32::{self, DerivationPath, Xpriv},
    key::{Keypair, Secp256k1},
};

use crate::{
    chain_trait::{AssetTracker, ChainTrait, Persistable, SecureKey},
    config::CONFIG,
};

pub struct BitcoinWallet {
    pub derived_addresses: Vec<BitcoinAddress>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitcoinAddress {
    pub label: String,
    pub derive_path: DerivePath,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Change {
    /// External chain is used for addresses that are meant to be visible outside of the wallet (e.g. for receiving payments)
    External = 0,
    /// Internal chain is used for addresses which are not meant to be visible outside of the wallet and is used for return transaction change
    Internal = 1,
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::External => write!(f, "0"),
            Change::Internal => write!(f, "1"),
        }
    }
}

impl From<u8> for Change {
    fn from(value: u8) -> Self {
        match value {
            0 => Change::External,
            1 => Change::Internal,
            _ => panic!("Invalid bitcoin address change: {}", value),
        }
    }
}

impl From<Change> for u8 {
    fn from(chain: Change) -> Self {
        chain as u8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivePath {
    pub change: Change,
    pub index: u32,
}

impl ToString for DerivePath {
    fn to_string(&self) -> String {
        format!("{}/{}", self.change, self.index)
    }
}

impl DerivePath {
    pub fn bip86_path(&self, network: Network) -> Result<DerivationPath, String> {
        let purpose = 86;
        let coin_type = match network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        let account = 0;
        let path = format!(
            "m/{purpose}'/{coin_type}'/{account}'/{}/{}",
            self.change as i32, self.index
        );
        DerivationPath::from_str(&path).map_err(|e| format!("fail to derive bip86_path: {e}"))
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct BitcoinUnlock {
    pub address: String,
}

pub struct Prk {
    xpriv: Xpriv,
}

impl Drop for Prk {
    fn drop(&mut self) {
        self.xpriv.private_key.non_secure_erase();
    }
}

impl SecureKey for Prk {
    type Material = Xpriv;

    fn expose(&self) -> &Self::Material {
        &self.xpriv
    }
}

impl BitcoinWallet {
    pub fn derive_scripts_of_interes(&self, xpriv: &Xpriv) -> Result<HashSet<ScriptBuf>, String> {
        let mut scripts_of_interes: HashSet<bip157::ScriptBuf> = HashSet::new();
        let network = CONFIG.bitcoin.network();

        for address in self.derived_addresses.iter() {
            let (_, address) = self
                .derive_child(xpriv, network, address.derive_path.clone())
                .map_err(|e| format!("derived bitcoin address corrupted {e}"))?;

            let scriptbuf = address.script_pubkey();
            scripts_of_interes.insert(scriptbuf);
        }

        Ok(scripts_of_interes)
    }

    pub fn derive_child(
        &self,
        xpriv: &Xpriv,
        network: Network,
        derive_path: DerivePath,
    ) -> Result<(Keypair, Address), String> {
        let secp = Secp256k1::new();
        // derive child private key
        let derive_path = &derive_path.bip86_path(network)?;
        let keypair = xpriv
            .derive_priv(&secp, derive_path)
            .map_err(|e| format!("Derivation error: {}", e))?
            .to_keypair(&secp);

        // x-only pubkey for taproot
        let (xonly_pk, _parity) = keypair.x_only_public_key();

        // Create taproot address (BIP341 tweak is done automatically by rust-bitcoin)
        let address = Address::p2tr(
            &secp, xonly_pk, None, // no script tree = BIP86 key-path spend
            network,
        );

        Ok((keypair, address))
    }

    pub fn is_deriviation_index_available(&self, derive_path: DerivePath) -> bool {
        !self
            .derived_addresses
            .iter()
            .any(|a| a.derive_path == derive_path)
    }

    pub fn unoccupied_deriviation_index(&self, change: Change) -> u32 {
        let occupied: HashSet<u32> = self
            .derived_addresses
            .iter()
            .filter(|a| a.derive_path.change == change)
            .map(|a| a.derive_path.index)
            .collect();
        (1..).find(|i| !occupied.contains(i)).unwrap_or(1)
    }

    pub fn add_child(&mut self, label: String, derive_path: DerivePath) {
        self.derived_addresses.push(BitcoinAddress {
            label,
            derive_path: derive_path,
        });
    }
}

impl ChainTrait for BitcoinWallet {
    type Prk = Prk;
    type UnlockResult = BitcoinUnlock;

    fn unlock(&self, prk: &Self::Prk) -> Result<Self::UnlockResult, String> {
        let main_receive_address = DerivePath {
            change: Change::External,
            index: 0,
        };
        let (_, btc_main_address) = self
            .derive_child(prk.expose(), CONFIG.bitcoin.network(), main_receive_address)
            .map_err(|e| e.to_string())?;

        Ok(BitcoinUnlock {
            address: btc_main_address.to_string(),
        })
    }
}

impl Persistable for BitcoinWallet {
    type Serialized = persistence::Wallet;

    fn serialize(&self) -> Result<Self::Serialized, String> {
        Ok(persistence::Wallet {
            childs: self
                .derived_addresses
                .iter()
                .map(|addr| persistence::ChildAddress {
                    label: addr.label.clone(),
                    purpose: addr.derive_path.change.clone().into(),
                    index: addr.derive_path.index,
                })
                .collect(),
        })
    }

    fn deserialize(data: Self::Serialized) -> Result<Self, String> {
        Ok(Self {
            derived_addresses: data
                .childs
                .into_iter()
                .map(|addr| BitcoinAddress {
                    label: addr.label,
                    derive_path: DerivePath {
                        change: Change::from(addr.purpose),
                        index: addr.index,
                    },
                })
                .collect(),
        })
    }
}

impl AssetTracker<BitcoinAddress> for BitcoinWallet {
    fn track(&mut self, address: BitcoinAddress) -> Result<(), String> {
        // Check if an address with the same purpose and index already exists
        if self
            .derived_addresses
            .iter()
            .any(|a| a.derive_path == address.derive_path)
        {
            return Err(format!(
                "Address with change {:?} and index {} already tracked",
                address.derive_path.change, address.derive_path.index
            ));
        }
        self.derived_addresses.push(address);
        Ok(())
    }

    fn untrack(&mut self, address: BitcoinAddress) -> Result<(), String> {
        let len_before = self.derived_addresses.len();
        self.derived_addresses
            .retain(|a| a.derive_path != address.derive_path);
        if self.derived_addresses.len() == len_before {
            return Err(format!("Address not tracked"));
        }
        Ok(())
    }

    fn list_tracked(&self) -> Vec<&BitcoinAddress> {
        self.derived_addresses.iter().collect()
    }
}

pub fn build_prk(mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
    let network = crate::config::CONFIG.bitcoin.network();
    let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed(passphrase);
    let xpriv = bip32::Xpriv::new_master(network, &seed).map_err(|e| e.to_string())?;
    Ok(Prk { xpriv })
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ChildAddress {
        pub label: String,
        pub purpose: u8,
        pub index: u32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Wallet {
        pub childs: Vec<ChildAddress>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unoccupied_deriviation_index() {
        let wallet = BitcoinWallet {
            derived_addresses: vec![
                BitcoinAddress {
                    label: "addr1".to_string(),
                    derive_path: DerivePath {
                        change: Change::External,
                        index: 1,
                    },
                },
                BitcoinAddress {
                    label: "addr2".to_string(),
                    derive_path: DerivePath {
                        change: Change::External,
                        index: 2,
                    },
                },
                BitcoinAddress {
                    label: "addr3".to_string(),
                    derive_path: DerivePath {
                        change: Change::External,
                        index: 19,
                    },
                },
                BitcoinAddress {
                    label: "change1".to_string(),
                    derive_path: DerivePath {
                        change: Change::Internal,
                        index: 0,
                    },
                },
            ],
        };

        assert_eq!(wallet.unoccupied_deriviation_index(Change::External), 3);
        assert_eq!(wallet.unoccupied_deriviation_index(Change::Internal), 1);
    }
}
