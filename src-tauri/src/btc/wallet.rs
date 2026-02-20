use std::collections::{HashMap, HashSet};

use bip39::Language;
use bitcoin::{
    Address, BlockHash, ScriptBuf, Wtxid,
    bip32::{self, Xpriv},
    hashes::Hash,
    key::{Keypair, Secp256k1},
};
use tokio::sync::mpsc;

use crate::{
    btc::{
        address::{Change, DerivePath, LabeledDerivationPath, Purpose},
        utxo::Utxo,
    },
    chain_trait::{AssetTracker, ChainTrait, Persistable, SecureKey},
    config::CONFIG,
};

#[derive(Default)]
pub struct RuntimeData {
    pub sync: sync::Sync,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DerivedScript {
    pub script: ScriptBuf,
    pub derive_path: DerivePath,
}

impl DerivedScript {
    pub fn new(script: ScriptBuf, derive_path: DerivePath) -> Self {
        Self {
            script,
            derive_path,
        }
    }
}

pub struct BitcoinWallet {
    pub derived_addresses: Vec<LabeledDerivationPath>,
    pub utxos: HashMap<String, Utxo>,
    pub cfilter_scanner_height: u32,
    pub initial_sync_done: bool,
    pub runtime: RuntimeData,
}

impl BitcoinWallet {
    pub fn default() -> BitcoinWallet {
        BitcoinWallet {
            cfilter_scanner_height: 1,
            initial_sync_done: false,
            derived_addresses: Vec::new(),
            utxos: HashMap::new(),
            runtime: RuntimeData::default(),
        }
    }

    pub fn build_prk(&self, mnemonic: &str, passphrase: &str) -> anyhow::Result<Prk> {
        let network = CONFIG.bitcoin.network();
        let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)?;
        let seed = mnemonic.to_seed(CONFIG.xprk_passphrase(passphrase));
        let xpriv = bip32::Xpriv::new_master(network.as_kind(), &seed)?;
        Ok(Prk { xpriv })
    }

    pub fn main_derive_path(&self) -> DerivePath {
        DerivePath {
            purpose: Purpose::Bip86,
            account: 0,
            network: CONFIG.bitcoin.network(),
            change: Change::External,
            index: 0,
        }
    }

    pub fn derive_scripts_of_interes(&self, prk: &Prk) -> anyhow::Result<HashSet<DerivedScript>> {
        let mut scripts_of_interes: HashSet<DerivedScript> = HashSet::new();
        {
            // Derive script for main receive script pubkey
            let derive_path = self.main_derive_path();
            let (_, address) = self.derive_child(prk.expose(), &derive_path)?;
            scripts_of_interes.insert(DerivedScript::new(address.script_pubkey(), derive_path));
        }

        for labled_derive_path in self.derived_addresses.iter() {
            let derive_path = labled_derive_path.derive_path.clone();
            let (_, address) = self.derive_child(prk.expose(), &derive_path)?;
            scripts_of_interes.insert(DerivedScript::new(address.script_pubkey(), derive_path));
        }

        Ok(scripts_of_interes)
    }

    pub fn derive_child(
        &self,
        xpriv: &Xpriv,
        derive_path: &DerivePath,
    ) -> anyhow::Result<(Keypair, Address)> {
        let secp = Secp256k1::new();
        // derive child private key
        let keypair = xpriv
            .derive_priv(&secp, &derive_path.to_path()?)?
            .to_keypair(&secp);

        // x-only pubkey for taproot
        let (xonly_pk, _parity) = keypair.x_only_public_key();
        let network: bitcoin::Network = CONFIG.bitcoin.network().into();
        // Create taproot address (BIP341 tweak is done automatically by rust-bitcoin)
        let address = Address::p2tr(
            &secp, xonly_pk, None, // no script tree = BIP86 key-path spend
            network,
        );

        Ok((keypair, address))
    }

    pub fn is_deriviation_path_free(&self, derive_path: DerivePath) -> bool {
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

    pub fn insert_utxos(&mut self, utxos: Vec<Utxo>) {
        for utxo in utxos {
            self.utxos.insert(utxo.id(), utxo);
        }
    }

    pub fn total_balance(&self) -> u64 {
        self.utxos
            .iter()
            .map(|utxo| utxo.1.output.value.to_sat())
            .sum()
    }

    pub fn add_script_of_interes(&mut self, script: DerivedScript) {
        self.runtime
            .sync
            .script_tx
            .as_ref()
            .expect("script_tx should be set after unlock")
            .send(script)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to send script of interest to channel: {}", e);
            });
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct BitcoinUnlock {
    pub address: String,
    pub total_balance: String,
}

pub struct UnlockCtx {}

impl ChainTrait for BitcoinWallet {
    type Prk = Prk;
    type UnlockResult = BitcoinUnlock;
    type UnlockContext = UnlockCtx;

    async fn unlock(
        &mut self,
        _: Self::UnlockContext,
        prk: &Self::Prk,
    ) -> Result<Self::UnlockResult, String> {
        let (_, btc_main_address) = self
            .derive_child(prk.expose(), &self.main_derive_path())
            .map_err(|e| e.to_string())?;

        Ok(BitcoinUnlock {
            address: btc_main_address.to_string(),
            total_balance: self.total_balance().to_string(),
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

pub mod sync {
    use super::*;

    #[derive(Default)]
    pub struct Sync {
        pub script_tx: Option<mpsc::UnboundedSender<DerivedScript>>,
        // pub result: Option<sync::Result>,
    }

    // #[derive(Clone)]
    // pub struct Result {
    //     pub update: bip157::SyncUpdate,
    //     #[allow(dead_code)]
    //     pub broadcast_min_fee_rate: bip157::FeeRate,
    //     #[allow(dead_code)]
    //     pub avg_fee_rate: bip157::FeeRate,
    // }

    // #[derive(Clone)]
    // pub enum Event {
    //     ChainSynced(Result),
    //     // BlockHeader(bip157::chain::IndexedHeader),
    //     NewUtxos(Vec<Utxo>),
    // }
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    use crate::btc::address::DerivePathSlice;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ChildAddress {
        pub label: String,
        pub devive_path: DerivePathSlice,
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
        pub deriviation_path: DerivePathSlice,
        /// Block height where this UTXO was created
        pub block_height: u32,
        /// Block hash for additional integrity
        pub block_hash: [u8; 32],
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Wallet {
        pub childs: Vec<ChildAddress>,
        pub utxos: Vec<Utxo>,
        pub cfilter_scanner_height: Option<u32>,
        pub initial_sync_done: bool,
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
                    let path = addr.derive_path.to_slice();
                    Ok(persistence::ChildAddress {
                        label: addr.label.clone(),
                        devive_path: path,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            utxos: self
                .utxos
                .values()
                .map(|utxo| persistence::Utxo {
                    block_hash: utxo.block.hash.to_byte_array(),
                    block_height: utxo.block.height,
                    deriviation_path: utxo.derive_path.to_slice(),
                    script_pubkey: utxo.output.script_pubkey.to_bytes(),
                    txid: utxo.tx_id.to_byte_array(),
                    value: utxo.output.value.to_sat(),
                    vout: utxo.vout,
                })
                .collect(),
            initial_sync_done: self.initial_sync_done,
            cfilter_scanner_height: Some(self.cfilter_scanner_height),
        })
    }

    fn deserialize(data: Self::Serialized) -> Result<Self, String> {
        let derived_addresses = data
            .childs
            .into_iter()
            .map(|addr| {
                let derive_path = DerivePath::from_slice(addr.devive_path)?;
                Ok(LabeledDerivationPath {
                    label: addr.label,
                    derive_path,
                })
            })
            .collect::<Result<Vec<LabeledDerivationPath>, String>>()?;
        let utxos: HashMap<String, Utxo> = data
            .utxos
            .iter()
            .map(|utxo| {
                let derive_path = DerivePath::from_slice(utxo.deriviation_path)?;
                let utxo = Utxo {
                    tx_id: Wtxid::from_byte_array(utxo.txid),
                    block: crate::btc::utxo::BlockHeader {
                        hash: BlockHash::from_byte_array(utxo.block_hash),
                        height: utxo.block_height,
                    },
                    vout: utxo.vout,
                    derive_path,
                    output: bitcoin::TxOut {
                        script_pubkey: ScriptBuf::from_bytes(utxo.script_pubkey.clone()),
                        value: bitcoin::Amount::from_sat(utxo.value),
                    },
                };
                Ok((utxo.id(), utxo))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            derived_addresses,
            utxos,
            initial_sync_done: data.initial_sync_done,
            cfilter_scanner_height: data.cfilter_scanner_height.unwrap_or(0),
            runtime: RuntimeData::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::btc::address::Purpose;

    use super::*;

    #[test]
    fn test_unoccupied_deriviation_index() {
        let network = CONFIG.bitcoin.network();
        let purpose = Purpose::Bip86;
        let wallet = BitcoinWallet {
            utxos: HashMap::new(),
            cfilter_scanner_height: 0,
            initial_sync_done: false,
            runtime: RuntimeData::default(),
            derived_addresses: vec![
                LabeledDerivationPath {
                    label: "addr1".to_string(),
                    derive_path: DerivePath {
                        purpose,
                        account: 0,
                        network,
                        change: Change::External,
                        index: 1,
                    },
                },
                LabeledDerivationPath {
                    label: "addr2".to_string(),
                    derive_path: DerivePath {
                        purpose,
                        account: 0,
                        network,
                        change: Change::External,
                        index: 2,
                    },
                },
                LabeledDerivationPath {
                    label: "addr3".to_string(),
                    derive_path: DerivePath {
                        purpose,
                        account: 0,
                        network,
                        change: Change::External,
                        index: 19,
                    },
                },
                LabeledDerivationPath {
                    label: "change1".to_string(),
                    derive_path: DerivePath {
                        purpose,
                        account: 0,
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
