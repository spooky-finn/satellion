use bip157::{BlockHash, Wtxid};
use bitcoin::TxOut;

use crate::btc::address::DerivePath;

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u32,
}

/// Unspent transaction output domain model
#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Wtxid,
    pub vout: usize,
    pub output: TxOut,
    pub derive_path: DerivePath,
    pub block: BlockHeader,
}

impl Utxo {
    pub fn id(&self) -> String {
        format!("{}{}", self.tx_id.to_string(), self.vout)
    }
}
