use crate::config::CONFIG;
use alloy::consensus::SignableTransaction;
use alloy::consensus::TxEnvelope;
use alloy::network::Ethereum;
use alloy::network::TransactionBuilder;
use alloy::network::TxSignerSync;
use alloy::primitives::FixedBytes;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::rpc::types::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use std::sync::Arc;
use tauri::Url;

pub fn new() -> Result<RootProvider, String> {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"),
    );
    Ok(provider)
}

#[derive(serde::Serialize, Debug, PartialEq)]
pub struct TxPresendInfo {
    pub gas_limit: u64,
    pub gas_price: u128,
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
        token_symbol: String,
        value: U256,
        sender: Address,
        recipient: Address,
    ) -> Result<TxPresendInfo, String> {
        if token_symbol != "ETH" {
            return Err("Only ETH is supported for now".to_string());
        }
        let tx_count = self.get_tx_count(sender).await?;
        let nonce = tx_count + 1;

        let tx = TransactionRequest::default()
            .with_to(recipient)
            .with_value(value)
            .with_nonce(nonce);

        let estimated_gas = self
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| format!("Failed to estimate gas: {e}"))?;

        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| format!("Failed to get gas price: {e}"))?;

        let estimator = self
            .provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| format!("Failed to estimate EIP-1559 fees: {e}"))?;

        let tx = tx
            .with_gas_price(gas_price)
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas);

        self.tx = Some(tx);
        Ok(TxPresendInfo {
            gas_limit: estimated_gas,
            gas_price: gas_price,
        })
    }

    pub async fn sign_and_send_tx(
        &mut self,
        signer: &PrivateKeySigner,
    ) -> Result<FixedBytes<32>, String> {
        let tx = self.tx.take().ok_or("Transaction not prepared")?;

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

        let token_symbol = "ETH".to_string();
        let value = U256::from(100);
        let recipient = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let sender = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let res = builder
            .eth_prepare_send_tx(token_symbol, value, sender, recipient)
            .await
            .unwrap();
        println!("{:?}", res);
    }
}
