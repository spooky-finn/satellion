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
pub struct BuildTxRequest {
    pub value: String,
    pub recipient: String,
    pub utxo_selection_method: UtxoSelectionStrategy,
}

#[derive(Type, Serialize)]
pub struct BuildTxResponse {
    pub fee: u32,
}

#[derive(Type, Deserialize)]
pub struct BroadcastTxRequest {}

#[derive(Type, Serialize)]
pub struct BroadcastTxResponse {
    pub tx_id: String,
}

#[derive(Type, Deserialize)]
pub struct BumpFeeRequest {
    /// Txid of the unconfirmed transaction whose fee we're bumping.
    pub parent_tx_id: String,
    /// sat/vB to pay on the *child* tx. With package-aware mining the parent
    /// is pulled in alongside; the package's effective rate is somewhere
    /// between this and the parent's own rate.
    pub target_fee_rate_sat_vb: f64,
}

#[derive(Type, Serialize)]
pub struct BumpFeeResponse {
    pub child_tx_id: String,
    pub child_fee: u32,
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddress {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[derive(Serialize, specta::Type)]
pub struct AccountSummary {
    pub index: AccountIndex,
    pub name: String,
}

#[derive(Serialize, specta::Type)]
pub struct BitcoinUnlock {
    pub accounts: Vec<AccountSummary>,
    pub active_account: ActiveAccountView,
}

#[derive(Type, Serialize)]
pub struct UtxoView {
    pub utxo_id: OutPointRef,
    pub value: String,
    pub deriv_path: String,
    pub address_label: Option<String>,
    pub confirmed: bool,
}

impl Utxo {
    pub fn to_view(&self, address_label_map: &KeyDerivationPathLabelMap) -> UtxoView {
        UtxoView {
            value: self.output.value.to_sat().to_string(),
            utxo_id: self.outpoint().into(),
            deriv_path: self.derivation.to_string(),
            address_label: self.label(address_label_map),
            confirmed: self.height > 0,
        }
    }
}

#[derive(Serialize, specta::Type)]
pub struct ActiveAccountView {
    pub index: u32,
    /** main external address to accept payments */
    pub address: String,
    pub total_balance: String,
    pub utxo: Vec<UtxoView>,
}

#[derive(Type, Serialize)]
pub struct DiscoveryReportView {
    /// Account indexes that have on-chain activity (existing or newly created).
    pub accounts: Vec<u32>,
    /// Number of new derivation paths added to the wallet.
    pub paths_added: u32,
    /// Number of new UTXOs added across all accounts.
    pub utxos_added: u32,
    /// Aggregate value (sat) of newly added UTXOs, serialized as a string to
    /// preserve precision across the IPC boundary.
    pub total_value_sat: String,
}

#[derive(Type, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct OutPointRef {
    pub tx_id: String,
    pub vout: u32,
}

impl std::fmt::Display for OutPointRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tx_id, self.vout)
    }
}

impl From<OutPoint> for OutPointRef {
    fn from(value: OutPoint) -> Self {
        Self {
            tx_id: value.txid.to_string(),
            vout: value.vout,
        }
    }
}

impl TryFrom<OutPointRef> for OutPoint {
    type Error = String;

    fn try_from(value: OutPointRef) -> Result<Self, Self::Error> {
        Ok(Self {
            txid: Txid::from_str(&value.tx_id).map_err(|e| format!("invalid txid {}", e))?,
            vout: value.vout,
        })
    }
}
