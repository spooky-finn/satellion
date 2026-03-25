use bitcoin::{BlockHash, TxOut, Txid};

use crate::btc::{
    account::SchemaLabelMap,
    key_derivation::{Change, KeyDerivationPath},
};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: BlockHash,
    pub height: u32,
}

pub type UtxoIdentifier = String;

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
    pub fn outpoint(&self) -> UtxoIdentifier {
        format!("{}{}", self.tx_id, self.vout)
    }

    pub fn label(&self, schema_label_map: &SchemaLabelMap) -> Option<String> {
        let label: Option<String> = match self.derivation.change {
            Change::Internal => Some("Change".to_string()),
            Change::External => schema_label_map.get(&self.derivation.to_slice()).cloned(),
        };

        label
    }
}

pub mod persistence {
    use crate::btc::{
        key_derivation::{KeyDerivationPath, KeyDeriviationPathSlice},
        utxo::Utxo,
    };
    use bitcoin::{Txid, hashes::Hash};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct UtxoData {
        /// Transaction hash (32 bytes)
        pub txid: [u8; 32],
        /// Output index within the transaction
        pub vout: usize,
        /// Value in satoshis
        pub value: u64,
        /// ScriptPubKey (raw hex)
        pub script_pubkey: Vec<u8>,
        /// BIP-84 path to derive priv key from xpriv key
        pub derivation: KeyDeriviationPathSlice,
        pub height: u32,
    }

    impl Utxo {
        pub fn serialize(&self) -> Result<UtxoData, String> {
            Ok(UtxoData {
                height: self.height,
                derivation: self.derivation.to_slice(),
                script_pubkey: self.output.script_pubkey.to_bytes(),
                txid: self.tx_id.to_byte_array(),
                value: self.output.value.to_sat(),
                vout: self.vout,
            })
        }
    }

    impl UtxoData {
        pub fn deserialize(&self) -> Result<Utxo, String> {
            Ok(Utxo {
                tx_id: Txid::from_byte_array(self.txid),
                height: self.height,
                vout: self.vout,
                derivation: KeyDerivationPath::from_slice(self.derivation)?,
                output: bitcoin::TxOut {
                    script_pubkey: bitcoin::ScriptBuf::from_bytes(self.script_pubkey.clone()),
                    value: bitcoin::Amount::from_sat(self.value),
                },
            })
        }
    }
}
