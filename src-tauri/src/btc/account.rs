use std::collections::{HashMap, HashSet};

use bitcoin::{Address, Network, address::NetworkChecked};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    btc::{
        Prk,
        key_derivation::{
            Change, ChildKeyDeriviationScheme, KeyDerivationPath, KeyDeriviationPathSlice,
        },
        utxo::{self, OutPointDto, Utxo},
    },
    chain_trait::{AccountIndex, SecureKey},
};

type AccountAddresses = Vec<ChildKeyDeriviationScheme>;

#[derive(Clone)]
pub struct Account {
    pub index: AccountIndex,
    pub name: String,
    pub addresses: AccountAddresses,
    pub utxos: HashMap<OutPointDto, Utxo>,
}

#[derive(Clone, Type, Deserialize)]
pub enum UtxoSelectionMethod {
    Manual(Vec<OutPointDto>),
    Automatic(u32),
}

#[derive(Serialize, specta::Type)]
pub struct ActiveAccountDto {
    /** main external address to accept payments */
    pub address: String,
    pub total_balance: String,
}

impl Account {
    pub fn new(network: Network, account: AccountIndex, name: String) -> Self {
        Self {
            index: account,
            name,
            addresses: vec![
                ChildKeyDeriviationScheme {
                    label: "main".to_string(),
                    path: KeyDerivationPath::new_bip86(network, account, Change::External, 0),
                },
                ChildKeyDeriviationScheme {
                    label: "main_change".to_string(),
                    path: KeyDerivationPath::new_bip86(network, account, Change::Internal, 0),
                },
            ],
            utxos: HashMap::new(),
        }
    }

    pub fn info(&self, prk: &Prk, network: Network) -> Result<ActiveAccountDto, String> {
        let main_key_path = KeyDerivationPath::new_bip86(network, self.index, Change::External, 0);
        let mainkey = main_key_path
            .derive(prk.expose())
            .map_err(|e| e.to_string())?;

        Ok(ActiveAccountDto {
            address: mainkey.address.to_string(),
            total_balance: self.total_balance().to_string(),
        })
    }

    pub fn is_deriviation_path_available(&self, path: KeyDerivationPath) -> bool {
        !self.addresses.iter().any(|a| a.path == path)
    }

    pub fn unoccupied_address(&self, change: Change) -> u32 {
        let occupied: HashSet<u32> = self
            .addresses
            .iter()
            .filter(|a| a.path.change == change)
            .map(|a| a.path.index)
            .collect();
        (1..).find(|i| !occupied.contains(i)).unwrap_or(1)
    }

    pub fn get_external_addresess(&self) -> impl Iterator<Item = &ChildKeyDeriviationScheme> {
        self.addresses
            .iter()
            .filter(|a| a.path.change == Change::External)
    }

    pub fn add_address(&mut self, child: ChildKeyDeriviationScheme) {
        self.addresses.push(child);
    }

    pub fn set_utxos(&mut self, utxos: Vec<Utxo>) {
        self.utxos.clear();

        for utxo in utxos {
            self.utxos.insert(utxo.outpoint(), utxo);
        }
    }

    pub fn total_balance(&self) -> u64 {
        self.utxos
            .iter()
            .map(|utxo| utxo.1.output.value.to_sat())
            .sum()
    }

    pub fn derive_path_label_map(&self) -> KeyDerivationPathLabelMap {
        self.addresses
            .iter()
            .map(|e| (e.path.to_slice(), e.label.clone()))
            .collect()
    }

    pub fn derive_address_path_map(&self, prk: &Prk) -> AddressPathMap {
        self.addresses
            .iter()
            .filter_map(|schema| {
                schema
                    .path
                    .derive(prk.expose())
                    .ok()
                    .map(|child| (child.address, schema.path.clone()))
            })
            .collect()
    }

    pub fn choose_utxo(&self, method: UtxoSelectionMethod) -> Vec<&Utxo> {
        match method {
            UtxoSelectionMethod::Manual(out_point_dtos) => self.manual_utxo_select(out_point_dtos),
            UtxoSelectionMethod::Automatic(min_value) => {
                self.automatic_utxo_selection(min_value as u64)
            }
        }
    }

    fn automatic_utxo_selection(&self, _min_value: u64) -> Vec<&Utxo> {
        // TODO: implement
        vec![]
    }

    fn manual_utxo_select(&self, selected_outpoints: Vec<utxo::OutPointDto>) -> Vec<&Utxo> {
        selected_outpoints
            .iter()
            .filter_map(|outpoint| self.utxos.get(outpoint))
            .collect()
    }
}

pub type KeyDerivationPathLabelMap = HashMap<KeyDeriviationPathSlice, String>;
pub type AddressPathMap = HashMap<Address<NetworkChecked>, KeyDerivationPath>;

pub mod persistence {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    use crate::btc::{
        account::{Account, AccountIndex},
        key_derivation::{ChildKeyDeriviationScheme, KeyDerivationPath, KeyDeriviationPathSlice},
        utxo::{OutPointDto, Utxo, persistence::UtxoData},
    };

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AccountSnapshot {
        pub name: String,
        pub index: AccountIndex,
        pub addresses: Vec<DerivationPathSnapshot>,
        pub utxos: Vec<UtxoData>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DerivationPathSnapshot {
        pub label: String,
        pub path: KeyDeriviationPathSlice,
    }

    impl Account {
        pub fn serialize(&self) -> Result<AccountSnapshot, String> {
            Ok(AccountSnapshot {
                name: self.name.clone(),
                index: self.index,
                addresses: self
                    .addresses
                    .iter()
                    .map(|addr| DerivationPathSnapshot {
                        label: addr.label.clone(),
                        path: addr.path.to_slice(),
                    })
                    .collect(),
                utxos: self
                    .utxos
                    .values()
                    .map(|utxo| utxo.serialize().unwrap())
                    .collect(),
            })
        }
    }

    impl AccountSnapshot {
        pub fn deserialize(&self) -> Result<Account, String> {
            let derived_addresses = self
                .addresses
                .iter()
                .map(|addr| {
                    let path = KeyDerivationPath::from_slice(addr.path)?;
                    Ok(ChildKeyDeriviationScheme {
                        label: addr.label.clone(),
                        path,
                    })
                })
                .collect::<Result<Vec<ChildKeyDeriviationScheme>, String>>()?;

            let utxos: HashMap<OutPointDto, Utxo> = self
                .utxos
                .iter()
                .map(|utxo| {
                    let utxo = utxo.deserialize().unwrap();
                    Ok((utxo.outpoint(), utxo))
                })
                .collect::<Result<_, String>>()?;

            Ok(Account {
                name: self.name.clone(),
                index: self.index,
                addresses: derived_addresses,
                utxos,
            })
        }
    }
}
