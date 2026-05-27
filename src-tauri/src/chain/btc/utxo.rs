use bitcoin::{BlockHash, OutPoint, TxOut, Txid};

use crate::chain::btc::{
    account::KeyDerivationPathLabelMap,
    key_derivation::{Change, KeyDerivationPath},
};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Txid,
    pub vout: u32,
    pub output: TxOut,
    pub derivation: KeyDerivationPath,
    pub height: u32,
}

impl Utxo {
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.tx_id,
            vout: self.vout,
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
