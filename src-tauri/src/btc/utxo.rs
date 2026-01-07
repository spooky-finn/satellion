use bip157::BlockHash;
use bitcoin::{TxOut, hashes::sha256d};

use crate::btc::address::DerivePath;

/// Unspent transaction output
pub struct UTxO {
    pub block_hash: BlockHash,
    pub block_height: u32,
    pub tx_hash: sha256d::Hash,
    pub output: TxOut,
    pub vout_idx: usize,
    pub derive_path: DerivePath,
}
