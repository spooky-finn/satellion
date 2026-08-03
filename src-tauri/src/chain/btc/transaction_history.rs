use crate::{
    repository::{BtcChainData, NewTx, TxDirection, TxStatus},
    transaction_history::{TransactionHistoryEntry, WalletTxContext},
};

pub struct BtcOutgoingTransfer {
    pub tx_hash: String,
    pub recipient: Option<String>,
    pub amount_sat: u64,
    pub fee_sat: u32,
    pub vsize: u32,
    pub change_value_sat: Option<u64>,
}

impl TransactionHistoryEntry for BtcOutgoingTransfer {
    fn tx_hash(&self) -> &str {
        &self.tx_hash
    }

    fn into_new_tx(self, context: WalletTxContext, created_at: i64) -> Result<NewTx, String> {
        let chain_data = BtcChainData {
            vsize: Some(self.vsize),
            rbf: true,
            parent_tx_id: None,
            change_value_sat: self.change_value_sat,
        };

        Ok(NewTx {
            wallet_name: context.wallet_name,
            chain: context.chain,
            account_index: context.account_index,
            tx_hash: self.tx_hash,
            direction: TxDirection::Outgoing,
            status: TxStatus::Pending,
            from_address: None,
            to_address: self.recipient,
            amount: self.amount_sat.to_string(),
            fee: Some(self.fee_sat.to_string()),
            block_height: None,
            chain_data: serde_json::to_value(chain_data)
                .map_err(|e| format!("encode bitcoin chain data: {e}"))?,
            created_at,
        })
    }
}
