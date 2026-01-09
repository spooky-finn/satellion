use std::collections::HashSet;

use bip39::Language;
use bip157::BlockHash;
pub use bitcoin::network::Network;
use bitcoin::{
    Address,
    bip32::{self, Xpriv},
    hashes::Hash,
    key::{Keypair, Secp256k1},
};

use crate::{
    btc::{
        address::{Change, DerivePath, LabeledDerivationPath},
        utxo::UTxO,
    },
    chain_trait::{AssetTracker, ChainTrait, Persistable, SecureKey},
    config::CONFIG,
};

pub struct BitcoinWallet {
    pub derived_addresses: Vec<LabeledDerivationPath>,
    pub utxos: Vec<UTxO>,
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DerivedScript {
    pub script: bip157::ScriptBuf,
    pub derive_path: DerivePath,
}

impl BitcoinWallet {
    pub fn derive_scripts_of_interes(
        &self,
        xpriv: &Xpriv,
    ) -> Result<HashSet<DerivedScript>, String> {
        let mut scripts_of_interes: HashSet<DerivedScript> = HashSet::new();
        let network = CONFIG.bitcoin.network();

        for labled_derive_path in self.derived_addresses.iter() {
            let derive_path = labled_derive_path.derive_path.clone();
            let (_, address) = self
                .derive_child(xpriv, network, derive_path.clone())
                .map_err(|e| format!("derived bitcoin address corrupted {e}"))?;
            scripts_of_interes.insert(DerivedScript {
                derive_path,
                script: address.script_pubkey(),
            });
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
        let derive_path = &derive_path.as_bip86_path()?;
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
        self.derived_addresses
            .push(LabeledDerivationPath { label, derive_path });
    }

    pub fn list_external_addresess(&self) -> impl Iterator<Item = &LabeledDerivationPath> {
        self.derived_addresses
            .iter()
            .filter(|a| a.derive_path.change == Change::External)
    }

    pub fn insert_utxos(&mut self, utxos: Vec<UTxO>) {
        self.utxos.extend(utxos);
    }
}

impl ChainTrait for BitcoinWallet {
    type Prk = Prk;
    type UnlockResult = BitcoinUnlock;

    fn unlock(&self, prk: &Self::Prk) -> Result<Self::UnlockResult, String> {
        let main_receive_address = DerivePath {
            network: CONFIG.bitcoin.network(),
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
                .map(|addr| {
                    let path = addr.derive_path.as_bip86_path()?.to_string();
                    Ok(persistence::ChildAddress {
                        label: addr.label.clone(),
                        devive_path: path,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            utxos: self
                .utxos
                .iter()
                .map(|each| persistence::Utxo {
                    block_hash: each.block.hash.to_byte_array(),
                    block_height: each.block.height,
                    deriviation_path: each.derive_path.to_string(),
                    script_pubkey: each.output.script_pubkey.to_bytes(),
                    txid: each.tx_id.to_byte_array(),
                    value: each.output.value.to_sat(),
                    vout: each.vout,
                })
                .collect(),
        })
    }

    fn deserialize(data: Self::Serialized) -> Result<Self, String> {
        let derived_addresses = data
            .childs
            .into_iter()
            .map(|addr| {
                let derive_path = DerivePath::from_str(&addr.devive_path)?;
                Ok(LabeledDerivationPath {
                    label: addr.label,
                    derive_path,
                })
            })
            .collect::<Result<Vec<LabeledDerivationPath>, String>>()?;
        let utxos = data
            .utxos
            .iter()
            .map(|utxo| {
                let derive_path = DerivePath::from_str(&utxo.deriviation_path)?;
                Ok(UTxO {
                    tx_id: Hash::from_byte_array(utxo.txid),
                    block: crate::btc::utxo::BlockHeader {
                        hash: BlockHash::from_byte_array(utxo.block_hash),
                        height: utxo.block_height,
                    },
                    vout: utxo.vout,
                    derive_path,
                    output: bitcoin::TxOut {
                        script_pubkey: bip157::ScriptBuf::from_bytes(utxo.script_pubkey.clone()),
                        value: bitcoin::Amount::from_sat(utxo.value),
                    },
                })
            })
            .collect::<Result<Vec<UTxO>, String>>()?;
        Ok(Self {
            derived_addresses,
            utxos,
        })
    }
}

impl AssetTracker<LabeledDerivationPath> for BitcoinWallet {
    fn track(&mut self, address: LabeledDerivationPath) -> Result<(), String> {
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

    fn untrack(&mut self, address: LabeledDerivationPath) -> Result<(), String> {
        let len_before = self.derived_addresses.len();
        self.derived_addresses
            .retain(|a| a.derive_path != address.derive_path);
        if self.derived_addresses.len() == len_before {
            return Err("Address not tracked".to_string());
        }
        Ok(())
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
        pub devive_path: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Utxo {
        /// Transaction hash (32 bytes)
        pub txid: [u8; 32],
        /// Output index within the transaction
        pub vout: usize,
        /// Value in satoshis
        pub value: u64,
        /// ScriptPubKey (raw hex)
        pub script_pubkey: Vec<u8>,
        /// BIP-84 path to derive priv key from xpriv key
        pub deriviation_path: String,
        /// Block height where this UTXO was created
        pub block_height: u32,
        /// Block hash for additional integrity
        pub block_hash: [u8; 32],
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Wallet {
        pub childs: Vec<ChildAddress>,
        pub utxos: Vec<Utxo>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unoccupied_deriviation_index() {
        let network = CONFIG.bitcoin.network();
        let wallet = BitcoinWallet {
            utxos: vec![],
            derived_addresses: vec![
                LabeledDerivationPath {
                    label: "addr1".to_string(),
                    derive_path: DerivePath {
                        network,
                        change: Change::External,
                        index: 1,
                    },
                },
                LabeledDerivationPath {
                    label: "addr2".to_string(),
                    derive_path: DerivePath {
                        network,
                        change: Change::External,
                        index: 2,
                    },
                },
                LabeledDerivationPath {
                    label: "addr3".to_string(),
                    derive_path: DerivePath {
                        network,
                        change: Change::External,
                        index: 19,
                    },
                },
                LabeledDerivationPath {
                    label: "change1".to_string(),
                    derive_path: DerivePath {
                        network,
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
