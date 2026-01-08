use bip157::{BlockHash, Wtxid};
use bitcoin::TxOut;

use crate::btc::address::DerivePath;

pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u32,
}

/// Unspent transaction output domain model
pub struct UTxO {
    pub tx_id: Wtxid,
    pub vout: usize,
    pub output: TxOut,
    pub derive_path: DerivePath,
    pub block: BlockHeader,
}
