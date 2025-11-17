use crate::eth::token::Token;
use alloy::{
    consensus::{SignableTransaction, TxEnvelope},
    network::{TransactionBuilder, TxSignerSync},
    primitives::{
        Address, FixedBytes, U256,
        utils::{format_units, parse_ether},
    },
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
};
use alloy_provider::DynProvider;
use alloy_signer_local::PrivateKeySigner;
use std::str::FromStr;

sol!(
    #[sol(rpc)]
    Erc20Contract,
    "src/eth/abi/erc20.json"
);

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub token: Token,
    pub raw_amount: String,
    pub sender: Address,
    pub recipient: Address,
}

#[derive(Debug)]
pub struct TransactionMetadata {
    pub estimated_gas: u64,
    pub max_fee_per_gas: u128,
    pub cost: String,
}

pub struct TxBuilder {
    provider: DynProvider,
    builder_factory: TransferBuilderFactory,
    pending_tx: Option<TransactionRequest>,
}

pub struct TransferBuilderFactory;

impl TxBuilder {
    pub fn new(provider: DynProvider) -> Self {
        Self {
            provider,
            pending_tx: None,
            builder_factory: TransferBuilderFactory,
        }
    }

    async fn get_tx_count(&self, address: Address) -> Result<u64, String> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(|e| format!("Failed to get transaction count: {e}"))
    }

    /// Method creates transafer transaction and store it in the memory
    /// to wait until user confirms and sign transaction later
    pub async fn eth_create_transfer(
        &mut self,
        req: TransferRequest,
    ) -> Result<TransactionMetadata, String> {
        let chain_id = self
            .provider
            .get_chain_id()
            .await
            .map_err(|e| format!("Failed to get chain id: {e}"))?;
        let nonce = self.get_tx_count(req.sender).await?;
        let context = TransferContext {
            provider: self.provider.clone(),
            chain_id,
            nonce,
        };

        let builder = self.builder_factory.create_builder(&req.token.symbol);
        let tx_info = builder.build_transaction(req, context).await?;

        self.pending_tx = Some(tx_info.transaction);
        Ok(tx_info.metadata)
    }

    pub async fn sign_and_send_tx(
        &mut self,
        signer: &PrivateKeySigner,
    ) -> Result<FixedBytes<32>, String> {
        let tx = self.pending_tx.take().ok_or("Transaction not prepared")?;
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

        self.pending_tx = None;
        let hash = pending_tx.tx_hash().clone();
        Ok(hash)
    }
}

fn parse_tx_amount(token_symbol: String, amount: String) -> Result<U256, String> {
    if token_symbol == "ETH" {
        return parse_ether(&amount).map_err(|e| e.to_string());
    }
    U256::from_str(&amount).map_err(|e| e.to_string())
}

#[derive(Debug)]
pub struct Build {
    pub transaction: TransactionRequest,
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone)]
pub struct TransferContext {
    pub provider: DynProvider,
    pub chain_id: u64,
    pub nonce: u64,
}

pub trait TransferBuilder {
    async fn build_transaction(
        &self,
        req: TransferRequest,
        ctx: TransferContext,
    ) -> Result<Build, String>;
}

pub enum TransferBuilderType {
    Ether(EtherTransferBuilder),
    Token(TokenTransferBuilder),
}

impl TransferBuilder for TransferBuilderType {
    async fn build_transaction(
        &self,
        req: TransferRequest,
        ctx: TransferContext,
    ) -> Result<Build, String> {
        match self {
            TransferBuilderType::Ether(b) => b.build_transaction(req, ctx).await,
            TransferBuilderType::Token(b) => b.build_transaction(req, ctx).await,
        }
    }
}

impl TransferBuilderFactory {
    pub fn create_builder(&self, token_symbol: &str) -> TransferBuilderType {
        match token_symbol.to_uppercase().as_str() {
            "ETH" => TransferBuilderType::Ether(EtherTransferBuilder),
            _ => TransferBuilderType::Token(TokenTransferBuilder {}),
        }
    }
}

pub struct EtherTransferBuilder;

impl TransferBuilder for EtherTransferBuilder {
    async fn build_transaction(
        &self,
        req: TransferRequest,
        ctx: TransferContext,
    ) -> Result<Build, String> {
        let tx_value = parse_tx_amount(req.token.symbol.clone(), req.raw_amount)?;

        let tx = TransactionRequest::default()
            .with_from(req.sender)
            .with_to(req.recipient)
            .with_value(tx_value)
            .with_chain_id(ctx.chain_id)
            .with_nonce(ctx.nonce);

        let balance = ctx
            .provider
            .get_balance(req.sender)
            .await
            .map_err(|e| format!("Failed to get balance: {e}"))?;

        let estimated_gas = ctx
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| format!("Failed to estimate gas: {e}"))?;

        let estimator = ctx
            .provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| format!("Failed to estimate EIP-1559 fees: {e}"))?;

        let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        let tx_amount = tx_value.saturating_add(fee_ceiling);
        let ether_tx_amount = format_units(fee_ceiling, "ether").map_err(|e| e.to_string())?;

        if balance < tx_amount {
            if balance < fee_ceiling {
                let formatted_balance =
                    format_units(balance, "ether").map_err(|e| e.to_string())?;
                return Err(format!(
                    "Insufficient funds: total balance is {}, but estimated fee cost is {}",
                    formatted_balance, ether_tx_amount
                ));
            } else {
                let possible_send_amount = balance.saturating_sub(fee_ceiling);
                return Err(format!(
                    "Insufficient funds: you can send a maximum of {}",
                    format_units(possible_send_amount, "ether").map_err(|e| e.to_string())?
                ));
            }
        }

        let final_tx = tx
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas)
            .with_gas_limit(estimated_gas);

        Ok(Build {
            transaction: final_tx,
            metadata: TransactionMetadata {
                estimated_gas,
                max_fee_per_gas: estimator.max_fee_per_gas,
                cost: ether_tx_amount,
            },
        })
    }
}

pub struct TokenTransferBuilder;

impl TransferBuilder for TokenTransferBuilder {
    async fn build_transaction(
        &self,
        req: TransferRequest,
        ctx: TransferContext,
    ) -> Result<Build, String> {
        let value = U256::from_str(req.raw_amount.as_str())
            .map_err(|e| format!("failed to parse amount {}", e))?;

        let contract =
            Erc20Contract::Erc20ContractInstance::new(req.token.address, ctx.provider.clone());

        let transfer_call = contract.transfer(req.recipient, value);
        let tx = transfer_call
            .into_transaction_request()
            .with_from(req.sender)
            .with_chain_id(ctx.chain_id)
            .with_nonce(ctx.nonce);

        let estimated_gas = ctx
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| format!("Failed to estimate gas: {e}"))?;

        let estimator = ctx
            .provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| format!("Failed to estimate EIP-1559 fees: {e}"))?;

        let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        let ether_fee_amount = format_units(fee_ceiling, "ether").map_err(|e| e.to_string())?;

        let final_tx = tx
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas)
            .with_gas_limit(estimated_gas);

        Ok(Build {
            transaction: final_tx,
            metadata: TransactionMetadata {
                estimated_gas,
                max_fee_per_gas: estimator.max_fee_per_gas,
                cost: ether_fee_amount,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{
        constants::mainnet::{ETH, USDT},
        new_provider, new_provider_anvil,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_eth_prepare_send_tx() {
        let provider = new_provider();
        let mut builder = TxBuilder::new(provider);

        let raw_amount = "0.01".to_string();
        let recipient = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let sender = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let res = builder
            .eth_create_transfer(TransferRequest {
                token: ETH.clone(),
                raw_amount,
                sender,
                recipient,
            })
            .await
            .unwrap();
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_polymorphic_transfer() {
        let provider = new_provider_anvil();
        let mut builder = TxBuilder::new(provider);

        let raw_amount = "0.01".to_string();
        let recipient = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let sender = Address::from_str("0xd8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap(); // Vitalik's address

        // Test ETH transfer using the new polymorphic system
        let res = builder
            .eth_create_transfer(TransferRequest {
                token: ETH.clone(),
                raw_amount: raw_amount.clone(),
                sender,
                recipient,
            })
            .await
            .unwrap();
        println!("ETH transfer result: {:?}", res);

        // Convert 0.01 USDC to raw units (6 decimals): 0.01 * 10^6 = 10000
        let token_amount = "10000".to_string();
        let res = builder
            .eth_create_transfer(TransferRequest {
                token: USDT.clone(),
                raw_amount: token_amount,
                sender,
                recipient,
            })
            .await
            .unwrap();
        println!("USDC transfer result: {:?}", res);
    }

    #[tokio::test]
    async fn test_eth_prepare_send_tokens() {
        let provider = new_provider_anvil();
        let block_number = provider.get_block_number().await.unwrap();
        println!("blocnum {}", block_number);

        let vitalik = Address::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();

        let balance = provider.get_balance(vitalik).await.unwrap();
        let formated_balance = format_units(balance, "eth").unwrap();
        print!("formated_balance {}", formated_balance);
        // // Create two users, Alice and Bob.
        // let accounts = provider.get_accounts().await?;
        // let alice = accounts[0];
        // let bob = accounts[1];

        // // Deploy the `ERC20Example` contract.
        // let contract = ERC20Example::deploy(provider).await?;

        // // Register the balances of Alice and Bob before the transfer.
        // let alice_before_balance = contract.balanceOf(alice).call().await?;
        // let bob_before_balance = contract.balanceOf(bob).call().await?;

        // // Transfer and wait for inclusion.
        // let amount = U256::from(100);
        // let tx_hash = contract.transfer(bob, amount).send().await?.watch().await?;

        // println!("Sent transaction: {tx_hash}");

        // // Register the balances of Alice and Bob after the transfer.
        // let alice_after_balance = contract.balanceOf(alice).call().await?;
        // let bob_after_balance = contract.balanceOf(bob).call().await?;

        // // Check the balances of Alice and Bob after the transfer.
        // assert_eq!(alice_before_balance - alice_after_balance, amount);
        // assert_eq!(bob_after_balance - bob_before_balance, amount);
    }
}
