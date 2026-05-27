use std::str::FromStr;

use bitcoin::{OutPoint, Txid};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    chain::btc::{
        account::{KeyDerivationPathLabelMap, UtxoSelectionStrategy},
        utxo::Utxo,
    },
    chain_trait::AccountIndex,
};

#[derive(Type, Deserialize)]
pub struct BuildTx {
    pub value: String,
    pub recipient: String,
    pub utxo_selection_method: UtxoSelectionStrategy,
}

#[derive(Type, Serialize)]
pub struct BuildTxResult {}

#[derive(Type, Deserialize)]
pub struct SendTx {}

#[derive(Type, Serialize)]
pub struct BroadcastResult {
    pub tx_id: String,
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddressDto {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[derive(Serialize, specta::Type)]
pub struct AccountMetaDto {
    pub index: AccountIndex,
    pub name: String,
}

#[derive(Serialize, specta::Type)]
pub struct BitcoinUnlockDto {
    pub accounts: Vec<AccountMetaDto>,
    pub active_account: ActiveAccountDto,
}

#[derive(Type, Serialize)]
pub struct UtxoDto {
    pub utxo_id: OutPointDto,
    pub value: String,
    pub deriv_path: String,
    pub address_label: Option<String>,
}

impl Utxo {
    pub fn to_dto(&self, address_label_map: &KeyDerivationPathLabelMap) -> UtxoDto {
        UtxoDto {
            value: self.output.value.to_sat().to_string(),
            utxo_id: self.outpoint().into(),
            deriv_path: self.derivation.to_string(),
            address_label: self.label(address_label_map),
        }
    }
}

#[derive(Serialize, specta::Type)]
pub struct ActiveAccountDto {
    pub index: u32,
    /** main external address to accept payments */
    pub address: String,
    pub total_balance: String,
    pub utxo: Vec<UtxoDto>,
}

#[derive(Type, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct OutPointDto {
    pub tx_id: String,
    pub vout: u32,
}

impl std::fmt::Display for OutPointDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tx_id, self.vout)
    }
}

impl From<OutPoint> for OutPointDto {
    fn from(value: OutPoint) -> Self {
        Self {
            tx_id: value.txid.to_string(),
            vout: value.vout,
        }
    }
}

impl TryFrom<OutPointDto> for OutPoint {
    type Error = String;

    fn try_from(value: OutPointDto) -> Result<Self, Self::Error> {
        Ok(Self {
            txid: Txid::from_str(&value.tx_id).map_err(|e| format!("invalid txid {}", e))?,
            vout: value.vout,
        })
    }
}
