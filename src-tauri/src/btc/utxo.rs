use bitcoin::{BlockHash, TxOut, Wtxid};

use crate::btc::{
    account::SchemaLabelMap,
    address::{Change, DeriviationSchema},
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
    pub tx_id: Wtxid,
    pub vout: usize,
    pub output: TxOut,
    pub deriviation_schema: DeriviationSchema,
    pub block: BlockHeader,
}

impl Utxo {
    pub fn id(&self) -> UtxoIdentifier {
        format!("{}{}", self.tx_id, self.vout)
    }

    pub fn label(&self, schema_label_map: &SchemaLabelMap) -> Option<String> {
        let label: Option<String> = match self.deriviation_schema.change {
            Change::Internal => Some("Change".to_string()),
            Change::External => schema_label_map
                .get(&self.deriviation_schema.to_slice())
                .cloned(),
        };

        label
    }
}

pub mod persistence {
    use crate::btc::{
        address::{DeriviationSchema, DeriviationSchemaSlice},
        utxo::Utxo,
    };
    use bitcoin::{BlockHash, Wtxid, hashes::Hash};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BlockHeaderData {
        /// Block height where this UTXO was created
        pub height: u32,
        /// Block hash for additional integrity
        pub hash: [u8; 32],
    }

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
        pub deriviation_path: DeriviationSchemaSlice,
        pub block: BlockHeaderData,
    }

    impl Utxo {
        pub fn serialize(&self) -> Result<UtxoData, String> {
            Ok(UtxoData {
                block: BlockHeaderData {
                    height: self.block.height,
                    hash: self.block.hash.to_byte_array(),
                },
                deriviation_path: self.deriviation_schema.to_slice(),
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
                tx_id: Wtxid::from_byte_array(self.txid),
                block: crate::btc::utxo::BlockHeader {
                    hash: BlockHash::from_byte_array(self.block.hash),
                    height: self.block.height,
                },
                vout: self.vout,
                deriviation_schema: DeriviationSchema::from_slice(self.deriviation_path)?,
                output: bitcoin::TxOut {
                    script_pubkey: bitcoin::ScriptBuf::from_bytes(self.script_pubkey.clone()),
                    value: bitcoin::Amount::from_sat(self.value),
                },
            })
        }
    }
}
