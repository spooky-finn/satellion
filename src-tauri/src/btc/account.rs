use std::collections::{HashMap, HashSet};

use bitcoin::{
    Address,
    bip32::Xpriv,
    key::{Keypair, Secp256k1},
};

use crate::{
    btc::{
        self,
        key_derivation::{
            Change, DerivedScript, KeyDerivationPath, KeyDeriviationPathSlice,
            LabeledDeriviationScheme, Proposal,
        },
        utxo::{Utxo, UtxoIdentifier},
    },
    chain_trait::SecureKey,
    config::CONFIG,
};

pub type AccountIndex = u32;
type AccountAddresses = Vec<LabeledDeriviationScheme>;

#[derive(Clone)]
pub struct Account {
    pub index: AccountIndex,
    pub name: String,
    pub addresses: AccountAddresses,
    pub utxos: HashMap<UtxoIdentifier, Utxo>,
}

impl Account {
    pub fn new(account: AccountIndex, name: String) -> Result<Self, String> {
        let account = Self {
            index: account,
            name,
            addresses: vec![
                LabeledDeriviationScheme {
                    label: "main".to_string(),
                    path: Account::new_deriviation_scheme_for_account(account, Change::External, 0),
                },
                LabeledDeriviationScheme {
                    label: "main_change".to_string(),
                    path: Account::new_deriviation_scheme_for_account(account, Change::Internal, 0),
                },
            ],
            utxos: HashMap::new(),
        };
        Ok(account)
    }

    pub fn derive_script(xpriv: &Xpriv, path: KeyDerivationPath) -> Result<DerivedScript, String> {
        let (_, address) = Account::derive_child(xpriv, &path)?;
        let script = DerivedScript::new(address.script_pubkey(), path);
        Ok(script)
    }

    pub fn get_scripts_hashset(&self, xpriv: &Xpriv) -> Result<HashSet<DerivedScript>, String> {
        let mut scripts_of_interes: HashSet<DerivedScript> = HashSet::new();

        for LabeledDeriviationScheme {
            path: derivation_path,
            ..
        } in self.addresses.iter().cloned()
        {
            scripts_of_interes.insert(Account::derive_script(xpriv, derivation_path)?);
        }

        Ok(scripts_of_interes)
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

    pub fn add_child(
        &mut self,
        prk: &btc::Prk,
        label: String,
        path: KeyDerivationPath,
    ) -> Result<(Keypair, Address), String> {
        let child = Account::derive_child(prk.expose(), &path)?;
        self.addresses
            .push(LabeledDeriviationScheme { label, path });
        Ok(child)
    }

    pub fn get_external_addresess(&self) -> impl Iterator<Item = &LabeledDeriviationScheme> {
        self.addresses
            .iter()
            .filter(|a| a.path.change == Change::External)
    }

    pub fn add_utxos(&mut self, utxos: Vec<Utxo>) {
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

    pub fn new_deriviation_scheme_for_account(
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

    pub fn derive_child(
        xpriv: &Xpriv,
        path: &KeyDerivationPath,
    ) -> Result<(Keypair, Address), String> {
        let secp = Secp256k1::new();
        // derive child private key
        let keypair = xpriv
            .derive_priv(&secp, &path.to_path()?)
            .map_err(|e| format!("Derivation error: {}", e))?
            .to_keypair(&secp);

        // x-only pubkey for taproot
        let (internal_key, _parity) = keypair.x_only_public_key();

        // Create taproot address (BIP341 tweak is done automatically by rust-bitcoin)
        let address = Address::p2tr(
            &secp,
            internal_key,
            None, // no script tree = BIP86 key-path spend
            CONFIG.bitcoin.network(),
        );

        Ok((keypair, address))
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
        key_derivation::{KeyDerivationPath, KeyDeriviationPathSlice, LabeledDeriviationScheme},
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
                    Ok(LabeledDeriviationScheme {
                        label: addr.label.clone(),
                        path,
                    })
                })
                .collect::<Result<Vec<LabeledDeriviationScheme>, String>>()?;
            let utxos: HashMap<String, Utxo> = self
                .utxos
                .iter()
                .map(|utxo| {
                    let utxo = utxo.deserialize().unwrap();
                    Ok((utxo.id(), utxo))
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
