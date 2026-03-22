use std::collections::{HashMap, HashSet};

use bitcoin::{
    Address,
    bip32::Xpriv,
    key::{Keypair, Secp256k1},
};

use crate::{
    btc::{
        self,
        address::{
            Change, DerivedScript, DeriviationSchema, DeriviationSchemaSlice,
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
                    schema: Account::new_deriviation_scheme_for_account(
                        account,
                        Change::External,
                        0,
                    ),
                },
                LabeledDeriviationScheme {
                    label: "main_change".to_string(),
                    schema: Account::new_deriviation_scheme_for_account(
                        account,
                        Change::Internal,
                        0,
                    ),
                },
            ],
            utxos: HashMap::new(),
        };
        Ok(account)
    }

    pub fn derive_script(
        xpriv: &Xpriv,
        schema: DeriviationSchema,
    ) -> Result<DerivedScript, String> {
        let (_, address) = Account::derive_child(xpriv, &schema)?;
        let schema = DerivedScript::new(address.script_pubkey(), schema);
        Ok(schema)
    }

    pub fn get_scripts_hashset(&self, xpriv: &Xpriv) -> Result<HashSet<DerivedScript>, String> {
        let mut scripts_of_interes: HashSet<DerivedScript> = HashSet::new();

        for LabeledDeriviationScheme { schema, .. } in self.addresses.iter().cloned() {
            scripts_of_interes.insert(Account::derive_script(xpriv, schema)?);
        }

        Ok(scripts_of_interes)
    }

    pub fn deriviation_schema_available(&self, schema: DeriviationSchema) -> bool {
        !self.addresses.iter().any(|a| a.schema == schema)
    }

    pub fn unoccupied_deriviation_index(&self, change: Change) -> u32 {
        let occupied: HashSet<u32> = self
            .addresses
            .iter()
            .filter(|a| a.schema.change == change)
            .map(|a| a.schema.index)
            .collect();
        (1..).find(|i| !occupied.contains(i)).unwrap_or(1)
    }

    pub fn add_child(
        &mut self,
        prk: &btc::Prk,
        label: String,
        schema: DeriviationSchema,
    ) -> Result<(Keypair, Address), String> {
        let child = Account::derive_child(prk.expose(), &schema)?;
        self.addresses
            .push(LabeledDeriviationScheme { label, schema });
        Ok(child)
    }

    pub fn get_external_addresess(&self) -> impl Iterator<Item = &LabeledDeriviationScheme> {
        self.addresses
            .iter()
            .filter(|a| a.schema.change == Change::External)
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
    ) -> DeriviationSchema {
        DeriviationSchema {
            purpose: Proposal::Bip86,
            network: CONFIG.bitcoin.network(),
            account,
            change,
            index,
        }
    }

    pub fn derive_child(
        xpriv: &Xpriv,
        schema: &DeriviationSchema,
    ) -> Result<(Keypair, Address), String> {
        let secp = Secp256k1::new();
        // derive child private key
        let keypair = xpriv
            .derive_priv(&secp, &schema.to_path()?)
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
            .map(|e| (e.schema.to_slice(), e.label.clone()))
            .collect()
    }
}

pub type SchemaLabelMap = HashMap<DeriviationSchemaSlice, String>;

pub mod persistence {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    use crate::btc::{
        account::{Account, AccountIndex},
        address::{DeriviationSchema, DeriviationSchemaSlice, LabeledDeriviationScheme},
        utxo::Utxo,
        utxo::persistence::UtxoData,
    };

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AccountSnapshot {
        pub name: String,
        pub index: AccountIndex,
        pub addresses: Vec<DeriviationSchemeSnapshot>,
        pub utxos: Vec<UtxoData>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct DeriviationSchemeSnapshot {
        pub label: String,
        pub deriviation_scheme: DeriviationSchemaSlice,
    }

    impl Account {
        pub fn serialize(&self) -> Result<AccountSnapshot, String> {
            Ok(AccountSnapshot {
                name: self.name.clone(),
                index: self.index,
                addresses: self
                    .addresses
                    .iter()
                    .map(|addr| DeriviationSchemeSnapshot {
                        label: addr.label.clone(),
                        deriviation_scheme: addr.schema.to_slice(),
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
                    let schema = DeriviationSchema::from_slice(addr.deriviation_scheme)?;
                    Ok(LabeledDeriviationScheme {
                        label: addr.label.clone(),
                        schema,
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
