use nakamoto::common::bitcoin::{BlockHash, OutPoint, TxOut, Txid};

use crate::btc::address::DerivePath;

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u64,
}

/// Unspent transaction output domain model
#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Txid,
    pub vout: u32,
    pub output: TxOut,
    pub derive_path: DerivePath,
    pub block: BlockHeader,
}

impl Utxo {
    pub fn out_point(&self) -> OutPoint {
        OutPoint::new(self.tx_id, self.vout)
    }
}
