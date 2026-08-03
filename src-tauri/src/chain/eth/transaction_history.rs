use alloy::primitives::utils::parse_units;

use super::transfer_builder::PendingTxMeta;
use crate::{
    repository::{EthChainData, NewTx, TxDirection, TxStatus},
    transaction_history::{TransactionHistoryEntry, WalletTxContext},
};

pub struct EthOutgoingTransfer {
    pub tx_hash: String,
    pub meta: PendingTxMeta,
}

impl TransactionHistoryEntry for EthOutgoingTransfer {
    fn tx_hash(&self) -> &str {
        &self.tx_hash
    }

    fn into_new_tx(self, context: WalletTxContext, created_at: i64) -> Result<NewTx, String> {
        let chain_data = EthChainData {
            nonce: self.meta.nonce,
            token_address: self.meta.payload.token.address.to_string(),
            token_symbol: self.meta.payload.token.symbol.clone(),
            token_decimals: self.meta.payload.token.decimals,
            max_fee_per_gas: self.meta.metadata_max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: self.meta.metadata_max_priority_fee_per_gas.to_string(),
            gas_limit: self.meta.metadata_estimated_gas.to_string(),
        };
        let amount = parse_units(
            &self.meta.payload.raw_amount,
            self.meta.payload.token.decimals,
        )
        .map(|u| u.get_absolute().to_string())
        .map_err(|e| format!("failed to encode ethereum transaction amount: {e}"))?;
        let fee_wei = self.meta.metadata_estimated_gas as u128 * self.meta.metadata_max_fee_per_gas;

        Ok(NewTx {
            wallet_name: context.wallet_name,
            chain: context.chain,
            account_index: context.account_index,
            tx_hash: self.tx_hash,
            direction: TxDirection::Outgoing,
            status: TxStatus::Pending,
            from_address: Some(self.meta.payload.sender.to_string()),
            to_address: Some(self.meta.payload.recipient.to_string()),
            amount,
            fee: Some(fee_wei.to_string()),
            block_height: None,
            chain_data: serde_json::to_value(chain_data)
                .map_err(|e| format!("encode ethereum chain data: {e}"))?,
            created_at,
        })
    }
}
