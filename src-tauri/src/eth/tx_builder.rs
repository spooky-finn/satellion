use crate::eth::token::Token;
use alloy::{
    consensus::{SignableTransaction, TxEnvelope},
    network::{TransactionBuilder, TxSignerSync},
    primitives::{
        Address, FixedBytes, U256,
        utils::{format_units, parse_ether, parse_units},
    },
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
};
use alloy_provider::{DynProvider, utils::Eip1559Estimation};
use alloy_signer_local::PrivateKeySigner;

sol!(
    #[sol(rpc)]
    Erc20Contract,
    "src/eth/abi/erc20.json"
);

const ETH_CHAIN_ID: u64 = 1;

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
    pub estimator: Eip1559Estimation,
    pub cost: String,
}

pub struct TxBuilder {
    provider: DynProvider,
    builder_factory: TransferBuilderFactory,
    pending_tx: Option<TransactionRequest>,
}

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

    /// Method creates transafer transaction for Ether or ERC20 tokens and store it in the session
    pub async fn create_transfer(
        &mut self,
        req: TransferRequest,
    ) -> Result<TransactionMetadata, String> {
        let nonce = self.get_tx_count(req.sender).await?;
        let ctx = TransferContext {
            provider: self.provider.clone(),
            nonce,
        };

        let builder = self.builder_factory.create_builder(&req.token.symbol);
        let tx_base = builder.build_transaction(&req, &ctx).await?;
        let build = self.calc_gas(tx_base).await?;

        match builder
            .check_balance(
                &req,
                &ctx,
                build.metadata.estimated_gas,
                build.metadata.estimator,
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Insufficient funds: {}", e));
            }
        };

        self.pending_tx = Some(build.transaction);
        Ok(build.metadata)
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

    async fn calc_gas(&self, tx: TransactionRequest) -> Result<Build, String> {
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

        let final_tx = tx
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas)
            .with_gas_limit(estimated_gas);

        let fee_ceiling: alloy::primitives::Uint<256, 4> =
            U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        let tx_fee_ether = format_units(fee_ceiling, "ether").map_err(|e| e.to_string())?;

        Ok(Build {
            transaction: final_tx.clone(),
            metadata: TransactionMetadata {
                estimator,
                estimated_gas,
                cost: tx_fee_ether,
            },
        })
    }
}

#[derive(Debug)]
pub struct Build {
    pub transaction: TransactionRequest,
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone)]
pub struct TransferContext {
    pub provider: DynProvider,
    pub nonce: u64,
}

pub trait TransferBuilder {
    async fn build_transaction(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
    ) -> Result<TransactionRequest, String>;
}

pub trait BalanceChecker {
    async fn check_balance(
        &self,
        _req: &TransferRequest,
        _ctx: &TransferContext,
        _estimated_gas: u64,
        _estimator: Eip1559Estimation,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub enum TransferBuilderType {
    Ether(EtherTransferBuilder),
    Token(TokenTransferBuilder),
}

impl TransferBuilder for TransferBuilderType {
    async fn build_transaction(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
    ) -> Result<TransactionRequest, String> {
        match self {
            TransferBuilderType::Ether(b) => b.build_transaction(req, ctx).await,
            TransferBuilderType::Token(b) => b.build_transaction(req, ctx).await,
        }
    }
}

impl BalanceChecker for TransferBuilderType {
    async fn check_balance(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
        estimated_gas: u64,
        estimator: Eip1559Estimation,
    ) -> Result<(), String> {
        match self {
            TransferBuilderType::Ether(b) => {
                b.check_balance(req, ctx, estimated_gas, estimator).await
            }
            TransferBuilderType::Token(b) => {
                b.check_balance(req, ctx, estimated_gas, estimator).await
            }
        }
    }
}

pub struct TransferBuilderFactory;

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
        req: &TransferRequest,
        ctx: &TransferContext,
    ) -> Result<TransactionRequest, String> {
        let value = parse_ether(&req.raw_amount).map_err(|e| e.to_string())?;
        let tx = TransactionRequest::default()
            .with_from(req.sender)
            .with_to(req.recipient)
            .with_value(value)
            .with_chain_id(ETH_CHAIN_ID)
            .with_nonce(ctx.nonce);
        Ok(tx)
    }
}

impl BalanceChecker for EtherTransferBuilder {
    async fn check_balance(
        &self,
        _req: &TransferRequest,
        _ctx: &TransferContext,
        _estimated_gas: u64,
        _estimator: Eip1559Estimation,
    ) -> Result<(), String> {
        // let balance = ctx
        //     .provider
        //     .get_balance(req.sender)
        //     .await
        //     .map_err(|e| format!("Failed to get balance: {e}"))?;

        // let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        // let tx_amount = tx_value.saturating_add(fee_ceiling);
        // let ether_tx_amount = format_units(fee_ceiling, "ether").map_err(|e| e.to_string())?;

        // if balance < tx_amount {
        //     if balance < fee_ceiling {
        //         let formatted_balance =
        //             format_units(balance, "ether").map_err(|e| e.to_string())?;
        //         return Err(format!(
        //             "Insufficient funds: total balance is {}, but estimated fee cost is {}",
        //             formatted_balance, ether_tx_amount
        //         ));
        //     } else {
        //         let possible_send_amount = balance.saturating_sub(fee_ceiling);
        //         return Err(format!(
        //             "Insufficient funds: you can send a maximum of {}",
        //             format_units(possible_send_amount, "ether").map_err(|e| e.to_string())?
        //         ));
        //     }
        // }
        Ok(())
    }
}

pub struct TokenTransferBuilder;

impl TransferBuilder for TokenTransferBuilder {
    async fn build_transaction(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
    ) -> Result<TransactionRequest, String> {
        let value = parse_units(&req.raw_amount, req.token.decimals)
            .map_err(|e| format!("failed to parse amount {}", e))?
            .get_absolute();
        let contract =
            Erc20Contract::Erc20ContractInstance::new(req.token.address, ctx.provider.clone());
        let transfer_call = contract.transfer(req.recipient, value);
        let tx = transfer_call
            .into_transaction_request()
            .with_from(req.sender)
            .with_chain_id(ETH_CHAIN_ID)
            .with_nonce(ctx.nonce);
        Ok(tx)
    }
}

impl BalanceChecker for TokenTransferBuilder {
    async fn check_balance(
        &self,
        _req: &TransferRequest,
        _ctx: &TransferContext,
        _estimated_gas: u64,
        _estimator: Eip1559Estimation,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{
        constants::{ETH, USDT},
        erc20_retriver::Erc20Retriever,
        new_provider_anvil,
    };
    use alloy_provider::{PendingTransactionConfig, ext::AnvilApi};
    use alloy_signer_local::LocalSigner;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_send_eth() {
        let provider = new_provider_anvil();
        let mut builder = TxBuilder::new(provider.clone());

        let raw_amount = "0.01".to_string(); // 0.01 ETH 
        let alice = LocalSigner::random();
        let bob = LocalSigner::random();
        // mint eth to alice wallet
        let alice_initial_balance = parse_ether("1").expect("invalid alice balance"); // 1 ETH
        provider
            .anvil_set_balance(alice.address(), alice_initial_balance)
            .await
            .expect("failed to mint ETH to alice");

        builder
            .create_transfer(TransferRequest {
                token: ETH.clone(),
                raw_amount,
                sender: alice.address(),
                recipient: bob.address(),
            })
            .await
            .unwrap();
        // sign and send tx
        let tx_hash = builder
            .sign_and_send_tx(&alice)
            .await
            .expect("failed to send tx");

        provider
            .watch_pending_transaction(PendingTransactionConfig::new(tx_hash))
            .await
            .unwrap();

        let bob_balance = provider
            .get_balance(bob.address())
            .await
            .expect("failed to get bob balance");
        assert_eq!(bob_balance, parse_ether("0.01").unwrap());
    }

    #[tokio::test]
    async fn test_send_tokens() {
        let provider = new_provider_anvil();
        let mut builder = TxBuilder::new(provider.clone());
        let erc20_retriver = Erc20Retriever::new(provider.clone());
        let token = USDT.clone();
        let sender = LocalSigner::random();
        let amount = "100.000000";

        match provider
            .anvil_set_balance(
                sender.address(),
                U256::from_str("10000000000000000000").unwrap(),
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!("Error setting balance: {}", e);
            }
        };
        provider
            .anvil_deal_erc20(
                sender.address(),
                token.address,
                parse_units(amount, token.decimals).unwrap().get_absolute(),
            )
            .await
            .unwrap();

        let recipient = LocalSigner::random();

        match builder
            .create_transfer(TransferRequest {
                token: USDT.clone(),
                raw_amount: amount.to_string(),
                sender: sender.address(),
                recipient: recipient.address(),
            })
            .await
        {
            Ok(tx) => {
                println!("tx {:?}", tx);
            }
            Err(e) => {
                panic!("Error creating transfer: {}", e);
            }
        };

        let tx_hash = builder.sign_and_send_tx(&sender).await.unwrap();
        let tx_receipt = provider
            .watch_pending_transaction(PendingTransactionConfig::new(tx_hash))
            .await
            .unwrap();
        println!("tx receipt {:?}", tx_receipt);
        let balance = erc20_retriver
            .balances(recipient.address(), vec![USDT.clone()])
            .await
            .unwrap();
        let token_balance = &balance[0];
        let recepien_balance = token_balance
            .token
            .get_balance(token_balance.balance)
            .to_string();
        assert_eq!(recepien_balance, amount);
    }
}
