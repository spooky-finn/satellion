use bitcoin::{BlockHash, TxOut, Txid};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::chain::btc::{
    account::KeyDerivationPathLabelMap,
    key_derivation::{Change, KeyDerivationPath},
};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u32,
}

/// Unspent transaction output domain model
#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Txid,
    pub vout: usize,
    pub output: TxOut,
    pub derivation: KeyDerivationPath,
    pub height: u32,
}

impl Utxo {
    pub fn outpoint(&self) -> OutPointDto {
        OutPointDto {
            tx_id: self.tx_id.to_string(),
            vout: self.vout.to_string(),
        }
    }

    pub fn label(&self, schema_label_map: &KeyDerivationPathLabelMap) -> Option<String> {
        let label: Option<String> = match self.derivation.change {
            Change::Internal => Some("Change".to_string()),
            Change::External => schema_label_map.get(&self.derivation.to_slice()).cloned(),
        };

        label
    }
}

#[derive(Type, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct OutPointDto {
    pub tx_id: String,
    pub vout: String,
}

impl std::fmt::Display for OutPointDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tx_id, self.vout)
    }
}

