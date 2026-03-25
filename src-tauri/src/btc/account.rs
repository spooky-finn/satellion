use std::collections::{HashMap, HashSet};

use crate::{
    btc::{
        key_derivation::{
            Change, ChildKeyDeriviationScheme, KeyDerivationPath, KeyDeriviationPathSlice, Proposal,
        },
        utxo::{Utxo, UtxoIdentifier},
    },
    chain_trait::AccountIndex,
    config::CONFIG,
};

type AccountAddresses = Vec<ChildKeyDeriviationScheme>;

#[derive(Clone)]
pub struct Account {
    pub index: AccountIndex,
    pub name: String,
    pub addresses: AccountAddresses,
    pub utxos: HashMap<UtxoIdentifier, Utxo>,
}

impl Account {
    pub fn new(account: AccountIndex, name: String) -> Self {
        Self {
            index: account,
            name,
            addresses: vec![
                ChildKeyDeriviationScheme {
                    label: "main".to_string(),
                    path: Account::new_deriviation_path(account, Change::External, 0),
                },
                ChildKeyDeriviationScheme {
                    label: "main_change".to_string(),
                    path: Account::new_deriviation_path(account, Change::Internal, 0),
                },
            ],
            utxos: HashMap::new(),
        }
    }

    pub fn deriviation_schema_available(&self, path: KeyDerivationPath) -> bool {
        !self.addresses.iter().any(|a| a.path == path)
    }

    pub fn unoccupied_deriviation_index(&self, change: Change) -> u32 {
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

    pub fn add_utxos(&mut self, utxos: Vec<Utxo>) {
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

    pub fn new_deriviation_path(
        account: AccountIndex,
        change: Change,
        index: u32,
    ) -> KeyDerivationPath {
        KeyDerivationPath {
            purpose: Proposal::Bip86,
            network: CONFIG.bitcoin.network(),
            account,
            change,
            index,
        }
    }

    pub fn schema_label_map(&self) -> SchemaLabelMap {
        self.addresses
            .iter()
            .map(|e| (e.path.to_slice(), e.label.clone()))
            .collect()
    }
}

pub type SchemaLabelMap = HashMap<KeyDeriviationPathSlice, String>;

pub mod persistence {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    use crate::btc::{
        account::{Account, AccountIndex},
        key_derivation::{ChildKeyDeriviationScheme, KeyDerivationPath, KeyDeriviationPathSlice},
        utxo::Utxo,
        utxo::persistence::UtxoData,
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
            let utxos: HashMap<String, Utxo> = self
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
