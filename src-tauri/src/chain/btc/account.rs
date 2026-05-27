use std::collections::{HashMap, HashSet};

use bitcoin::{Address, Network, OutPoint, address::NetworkChecked};
use serde::Deserialize;
use specta::Type;

use crate::{
    chain::btc::{
        Prk,
        dtos::OutPointDto,
        key_derivation::{
            Change, Child, KeyDerivationPath, KeyDeriviationPathSlice, LabeledKeyDerivationPath,
            Proposal,
        },
        utxo::Utxo,
    },
    chain_trait::{AccountIndex, SecureKey},
};

#[derive(Clone)]
pub struct Account {
    pub index: AccountIndex,
    pub name: String,
    pub keychain: KeyChain,
    pub utxo_set: UtxoSet,
}

impl Account {
    pub fn new(network: Network, account: AccountIndex, name: String) -> Self {
        Self {
            index: account,
            name,
            keychain: KeyChain::default(network, account),
            utxo_set: UtxoSet {
                entries: HashMap::new(),
            },
        }
    }

    pub fn main_key(
        &self,
        prk: &Prk,
        network: Network,
    ) -> Result<(Child, KeyDerivationPath), String> {
        let main_key_derive_path =
            KeyDerivationPath::new(Proposal::Bip86, network, self.index, Change::External, 0);
        let child = main_key_derive_path
            .derive(prk.expose())
            .map_err(|e| e.to_string())?;
        Ok((child, main_key_derive_path))
    }

    pub fn derive_address_path_map(&self, prk: &Prk, network: Network) -> AddressPathMap {
        let main_addr = self
            .main_key(prk, network)
            .expect("failed to derive main key");
        let mut map: AddressPathMap = self
            .keychain
            .paths
            .iter()
            .filter_map(|schema| {
                schema
                    .path
                    .derive(prk.expose())
                    .ok()
                    .map(|child| (child.taproot_address, schema.path.clone()))
            })
            .collect();
        map.insert(main_addr.0.taproot_address, main_addr.1);
        map
    }
}

#[derive(Clone)]
pub struct KeyChain {
    pub paths: Vec<LabeledKeyDerivationPath>,
}

impl KeyChain {
    fn default(network: Network, account: AccountIndex) -> Self {
        Self {
            paths: vec![LabeledKeyDerivationPath {
                label: "main".to_string(),
                path: KeyDerivationPath::new(
                    Proposal::Bip86,
                    network,
                    account,
                    Change::External,
                    0,
                ),
            }],
        }
    }

    /// Returns the first index that hasn't been used yet for a specific change type.
    pub fn next_unused_index(&self, change: Change) -> u32 {
        let occupied: HashSet<u32> = self
            .paths_by_change(&change)
            .map(|a| a.path.index)
            .collect();

        (1..).find(|i| !occupied.contains(i)).unwrap_or(0)
    }

    /// Checks if a specific path is already present in the keychain.
    pub fn contains_path(&self, path: KeyDerivationPath) -> bool {
        self.paths.iter().any(|a| a.path == path)
    }

    /// Returns an iterator of paths belonging to the internal (change) or external chain.
    pub fn paths_by_change(
        &self,
        change: &Change,
    ) -> impl Iterator<Item = &LabeledKeyDerivationPath> {
        self.paths.iter().filter(|a| a.path.change == *change)
    }

    /// Creates a lookup map of raw path slices to their respective labels.
    pub fn to_label_map(&self) -> KeyDerivationPathLabelMap {
        self.paths
            .iter()
            .map(|e| (e.path.to_slice(), e.label.clone()))
            .collect()
    }

    /// Registers a new path in the keychain.
    pub fn push(&mut self, child: LabeledKeyDerivationPath) {
        self.paths.push(child);
    }
}

#[derive(Clone)]
pub struct UtxoSet {
    /// A map of outpoints to their corresponding UTXO data.
    pub entries: HashMap<OutPoint, Utxo>,
}

#[derive(Clone, Type, Deserialize)]
pub enum UtxoSelectionStrategy {
    Manual(Vec<OutPointDto>),
    Auto(u32),
}

impl UtxoSet {
    /// Replaces the current set of UTXOs with a new collection.
    pub fn replace_all(&mut self, utxos: Vec<Utxo>) {
        self.entries.clear();
        self.entries
            .extend(utxos.into_iter().map(|u| (u.outpoint(), u)));
    }

    /// Calculates the sum of all unspent outputs in satoshis.
    pub fn total_value(&self) -> u64 {
        self.entries
            .iter()
            .map(|utxo| utxo.1.output.value.to_sat())
            .sum()
    }

    /// Selects a subset of UTXOs based on the provided strategy.
    pub fn select(&self, method: UtxoSelectionStrategy) -> Vec<&Utxo> {
        match method {
            UtxoSelectionStrategy::Manual(out_point_dtos) => {
                let outpoins = out_point_dtos
                    .into_iter()
                    .map(|e| e.try_into())
                    .collect::<Result<_, String>>()
                    .unwrap();
                self.select_by_outpoints(outpoins)
            }
            UtxoSelectionStrategy::Auto(min_value) => self.select_automatically(min_value as u64),
        }
    }

    /// Implementation of automated coin selection (e.g., Branch and Bound or Knapsack).
    fn select_automatically(&self, _min_value: u64) -> Vec<&Utxo> {
        // TODO: implement
        vec![]
    }

    /// Retrieves specific UTXOs by their outpoints, ignoring any that aren't in this set.
    fn select_by_outpoints(&self, selected_outpoints: Vec<OutPoint>) -> Vec<&Utxo> {
        selected_outpoints
            .iter()
            .filter_map(|outpoint| self.entries.get(outpoint))
            .collect()
    }
}

pub type KeyDerivationPathLabelMap = HashMap<KeyDeriviationPathSlice, String>;
pub type AddressPathMap = HashMap<Address<NetworkChecked>, KeyDerivationPath>;
