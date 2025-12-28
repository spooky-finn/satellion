use std::{collections::HashSet, str::FromStr};

use bip39::Language;
use bip157::ScriptBuf;
pub use bitcoin::network::Network;
use bitcoin::{
    Address,
    bip32::{self, DerivationPath, Xpriv},
    key::{Keypair, Secp256k1},
};

use crate::config::CONFIG;

/// Bitcoin-specific wallet data
pub struct WalletData {
    pub derived_addresses: Vec<BitcoinAddress>,
}

/// Represents a child derived address
#[derive(Debug, Clone, PartialEq)]
pub struct BitcoinAddress {
    pub label: String,
    pub purpose: AddressPurpose,
    pub index: u32,
}

#[derive(serde::Serialize, specta::Type)]
pub struct BitcoinUnlock {
    pub address: String,
}

pub struct Prk {
    pub xpriv: Xpriv,
}

impl Drop for Prk {
    fn drop(&mut self) {
        self.xpriv.private_key.non_secure_erase();
    }
}

impl WalletData {
    pub fn derive_scripts_of_interes(&self, xpriv: &Xpriv) -> Result<HashSet<ScriptBuf>, String> {
        let mut scripts_of_interes: HashSet<bip157::ScriptBuf> = HashSet::new();
        let network = CONFIG.bitcoin.network();

        for ba in self.derived_addresses.iter() {
            let (_, address) = self
                .derive_child(xpriv, network, ba.purpose.clone(), ba.index)
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
        purpose: AddressPurpose,
        index: u32,
    ) -> Result<(Keypair, Address), String> {
        let secp = Secp256k1::new();
        let path = create_diriviation_path(network, purpose, index);

        // derive child private key
        let keypair = xpriv
            .derive_priv(&secp, &path)
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

    pub fn unlock(&self, xpriv: &Xpriv) -> Result<BitcoinUnlock, String> {
        let (_, btc_main_address) = self
            .derive_child(xpriv, CONFIG.bitcoin.network(), AddressPurpose::Receive, 0)
            .map_err(|e| e.to_string())?;

        Ok(BitcoinUnlock {
            address: btc_main_address.to_string(),
        })
    }

    pub fn is_deriviation_index_available(&self, purpose: AddressPurpose, index: u32) -> bool {
        !self
            .derived_addresses
            .iter()
            .any(|a| a.index == index && a.purpose == purpose)
    }

    pub fn unoccupied_deriviation_index(&self, purpose: AddressPurpose) -> u32 {
        let occupied: HashSet<u32> = self
            .derived_addresses
            .iter()
            .filter(|a| a.purpose == purpose)
            .map(|a| a.index)
            .collect();

        (1..).find(|i| !occupied.contains(i)).unwrap_or(1)
    }

    pub fn add_child(&mut self, label: String, purpose: AddressPurpose, index: u32) {
        self.derived_addresses.push(BitcoinAddress {
            label,
            purpose,
            index,
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressPurpose {
    Receive = 0,
    Change = 1,
}

impl From<u8> for AddressPurpose {
    fn from(value: u8) -> Self {
        match value {
            0 => AddressPurpose::Receive,
            1 => AddressPurpose::Change,
            _ => panic!("Invalid bitcoin address purpose: {}", value),
        }
    }
}

impl From<AddressPurpose> for u8 {
    fn from(chain: AddressPurpose) -> Self {
        chain as u8
    }
}

pub fn derive_prk(mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
    let network = crate::config::CONFIG.bitcoin.network();
    let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed(passphrase);
    let xpriv = bip32::Xpriv::new_master(network, &seed).map_err(|e| e.to_string())?;
    Ok(Prk { xpriv })
}

pub fn create_diriviation_path(
    network: Network,
    purpose: AddressPurpose,
    address_index: u32,
) -> DerivationPath {
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let change = match purpose {
        AddressPurpose::Receive => 0,
        AddressPurpose::Change => 1,
    };

    let account = 0;
    let path = format!("m/86'/{coin_type}'/{account}'/{change}/{address_index}");
    DerivationPath::from_str(&path).expect("invalid child deriviation path")
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct ChildAddress {
        pub label: String,
        pub purpose: u8,
        pub index: u32,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct BitcoinData {
        pub childs: Vec<ChildAddress>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unoccupied_deriviation_index() {
        let wallet = WalletData {
            derived_addresses: vec![
                BitcoinAddress {
                    label: "addr1".to_string(),
                    purpose: AddressPurpose::Receive,
                    index: 1,
                },
                BitcoinAddress {
                    label: "addr2".to_string(),
                    purpose: AddressPurpose::Receive,
                    index: 2,
                },
                BitcoinAddress {
                    label: "addr3".to_string(),
                    purpose: AddressPurpose::Receive,
                    index: 19,
                },
                BitcoinAddress {
                    label: "change1".to_string(),
                    purpose: AddressPurpose::Change,
                    index: 0,
                },
            ],
        };

        assert_eq!(
            wallet.unoccupied_deriviation_index(AddressPurpose::Receive),
            3
        );
        assert_eq!(
            wallet.unoccupied_deriviation_index(AddressPurpose::Change),
            1
        );
    }
}
