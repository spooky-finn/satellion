use bitcoin::{TxOut, hashes::sha256d};

/// Unspent transaction output
pub struct UTxO {
    pub tx_hash: sha256d::Hash,
    pub output: TxOut,
    pub vout_idx: usize,
}
