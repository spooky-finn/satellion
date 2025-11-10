use crate::config::CONFIG;
use crate::ethereum::constants::token_symbol::TokenSymbol;
use alloy::consensus::{SignableTransaction, TxEnvelope};
use alloy::network::{Ethereum, TransactionBuilder, TxSignerSync};
use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, FixedBytes, U256, utils::parse_ether};
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use std::str::FromStr;
use std::sync::Arc;
use tauri::Url;

pub fn new() -> Result<RootProvider, String> {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"),
    );
    Ok(provider)
}

fn parse_tx_amount(token_symbol: TokenSymbol, amount: String) -> Result<U256, String> {
    if token_symbol == TokenSymbol::ETH {
        return parse_ether(&amount).map_err(|e| e.to_string());
    }
    U256::from_str(&amount).map_err(|e| e.to_string())
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct TxPresendInfo {
    pub estimated_gas: u64,
    pub max_fee_per_gas: u128,
    pub cost: String,
}

pub struct TxBuilder {
    provider: Arc<RootProvider<Ethereum>>,
    tx: Option<TransactionRequest>,
}

impl TxBuilder {
    pub fn new() -> Self {
        let provider = new().unwrap();
        Self {
            provider: Arc::new(provider),
            tx: None,
        }
    }

    pub async fn get_tx_count(&self, address: Address) -> Result<u64, String> {
        let count = self
            .provider
            .get_transaction_count(address)
            .await
            .map_err(|e| format!("Failed to get transaction count: {e}"))?;
        Ok(count)
    }

    pub async fn eth_prepare_send_tx(
        &mut self,
        token_symbol: TokenSymbol,
        raw_amount: String,
        sender: Address,
        recipient: Address,
    ) -> Result<TxPresendInfo, String> {
        if token_symbol != TokenSymbol::ETH {
            return Err("Only ETH is supported for now".to_string());
        }
        let nonce = self.get_tx_count(sender).await?;
        let tx_value = parse_tx_amount(token_symbol, raw_amount)?;
        let chain_id = self
            .provider
            .get_chain_id()
            .await
            .expect("Failed to get chain id");
        let tx = TransactionRequest::default()
            .with_from(sender)
            .with_to(recipient)
            .with_value(tx_value)
            .with_chain_id(chain_id)
            .with_nonce(nonce);
        let balance = self
            .provider
            .get_balance(sender)
            .await
            .map_err(|e| format!("Failed to get balance: {e}"))?;
        let estimated_gas = self
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| format!("Failed to estimate gas: {e}"))?;
        let estimator = self
            .provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| format!("Failed to estimate EIP-1559 fees: {e}"))?;

        let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        let tx_amount = tx_value.saturating_add(fee_ceiling);
        let ether_tx_amount = format_units(fee_ceiling, "ether").map_err(|e| e.to_string())?;

        if balance < tx_amount {
            // tx_amount = tx_value + fee_ceiling
            // If user can't afford full tx_value + fees, let's see if they can afford at least the fees
            if balance < fee_ceiling {
                // Can't even afford the fees, balance < estimated required gas * max fee per gas
                let formatted_balance =
                    format_units(balance, "ether").map_err(|e| e.to_string())?;
                return Err(format!(
                    "Insufficient funds: total balance is {}, but estimated fee cost is {}",
                    formatted_balance, ether_tx_amount
                ));
            } else {
                // Can afford fees, but not full tx_value. Suggest max possible ETH value to send
                let possible_send_amount = balance.saturating_sub(fee_ceiling);
                return Err(format!(
                    "Insufficient funds: you can send a maximum of {}",
                    format_units(possible_send_amount, "ether").map_err(|e| e.to_string())?
                ));
            }
        }

        let tx = tx
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas)
            .with_gas_limit(estimated_gas);

        self.tx = Some(tx);

        Ok(TxPresendInfo {
            estimated_gas,
            max_fee_per_gas: estimator.max_fee_per_gas,
            cost: ether_tx_amount,
        })
    }

    pub async fn sign_and_send_tx(
        &mut self,
        signer: &PrivateKeySigner,
    ) -> Result<FixedBytes<32>, String> {
        let tx = self.tx.take().ok_or("Transaction not prepared")?;
        // Check if the transaction is complete and return an error if not
        let is_complete = tx.complete_1559();
        if let Err(err) = is_complete {
            let messages = err.to_vec().join(", ");
            return Err(format!("Transaction is incomplete: {}", messages));
        }
        let mut tx_eip1559 = tx.build_1559().map_err(|e| e.to_string())?;

        let signature = signer
            .sign_transaction_sync(&mut tx_eip1559)
            .map_err(|e| format!("Failed to sign transaction: {e}"))?;

        let tx_envelope = TxEnvelope::Eip1559(tx_eip1559.into_signed(signature));
        let pending_tx = self
            .provider
            .send_tx_envelope(tx_envelope)
            .await
            .map_err(|e| format!("Failed to send transaction: {e}"))?;

        self.tx = None;
        let hash = pending_tx.tx_hash().clone();
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_eth_prepare_send_tx() {
        let mut builder = TxBuilder::new();

        let value = "100".to_string();
        let recipient = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let sender = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let res = builder
            .eth_prepare_send_tx(TokenSymbol::ETH, value, sender, recipient)
            .await
            .unwrap();
        println!("{:?}", res);
    }
}
