//! Ethereum transaction builder module.
//!
//! This module provides functionality for building and sending token transfers to the Ethereum network,
//! including both ETH transfers and ERC20 token transfers. It includes utilities for
//! gas estimation, balance checking, and transaction signing/broadcasting.
//!
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

/// Custom error type for balance checking failures.
#[derive(Debug, PartialEq)]
pub enum TransferBuilderError {
    /// Insufficient ETH balance for the requested transfer amount.
    ///
    /// # Fields
    /// - `current_balance`: User's current ETH balance in formatted string (e.g., "0.5")
    /// - `max_sendable`: Maximum ETH amount that can be sent after deducting gas fees
    InsufficientEther {
        current_balance: String,
        max_sendable: String,
    },
    /// Insufficient ETH balance to even pay for gas fees.
    ///
    /// This error occurs when the user's ETH balance is lower than the estimated
    /// gas fees required to execute any transaction.
    ///
    /// # Fields
    /// - `current_balance`: User's current ETH balance in formatted string (e.g., "0.00001")
    /// - `estimated_fee`: Estimated gas fees required for the transaction
    InsufficientGas {
        current_balance: String,
        estimated_fee: String,
    },
    /// Insufficient ERC20 token balance for the requested transfer.
    InsufficientTokens,
    /// Network or API error when querying account balance.
    NodeQuery(String),
    /// Error parsing amount strings into numeric values.
    AmountParse(String),
}

impl std::fmt::Display for TransferBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferBuilderError::InsufficientEther { max_sendable, .. } => {
                write!(
                    f,
                    "Insufficient ETH: you can send a maximum of {}",
                    max_sendable
                )
            }
            TransferBuilderError::InsufficientGas {
                current_balance,
                estimated_fee,
            } => {
                write!(
                    f,
                    "Insufficient ether for gas: ETH balance is {}, but estimated fee cost is {}",
                    current_balance, estimated_fee
                )
            }
            TransferBuilderError::InsufficientTokens { .. } => {
                write!(f, "Not enough tokens")
            }
            TransferBuilderError::NodeQuery(msg) => {
                write!(f, "Failed to query node: {}", msg)
            }
            TransferBuilderError::AmountParse(msg) => {
                write!(f, "Failed to parse amount: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransferBuilderError {}

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
    transfer_builder_factory: TransferBuilderFactory,
    pending_tx: Option<TransactionRequest>,
}

impl TxBuilder {
    pub fn new(provider: DynProvider) -> Self {
        Self {
            provider,
            pending_tx: None,
            transfer_builder_factory: TransferBuilderFactory,
        }
    }

    async fn get_tx_count(&self, address: Address) -> Result<u64, TransferBuilderError> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))
    }

    /// Method creates transafer transaction for Ether or ERC20 tokens and store it in the session
    pub async fn create_transfer(
        &mut self,
        req: TransferRequest,
    ) -> Result<TransactionMetadata, TransferBuilderError> {
        let nonce = self.get_tx_count(req.sender).await?;
        let ctx = TransferContext {
            provider: self.provider.clone(),
            nonce,
        };
        let transfer_builder = self
            .transfer_builder_factory
            .create_builder(&req.token.symbol);
        let tx_base = transfer_builder.build_transaction(&req, &ctx).await?;

        let Build {
            transaction,
            metadata,
        } = self.calc_gas(tx_base).await?;
        transfer_builder
            .check_balance(&req, &ctx, metadata.estimated_gas, metadata.estimator)
            .await?;
        self.pending_tx = Some(transaction);
        Ok(metadata)
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

    async fn calc_gas(&self, tx: TransactionRequest) -> Result<Build, TransferBuilderError> {
        let estimated_gas = self
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))?;

        let estimator = self
            .provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))?;

        let final_tx = tx
            .with_max_fee_per_gas(estimator.max_fee_per_gas)
            .with_max_priority_fee_per_gas(estimator.max_priority_fee_per_gas)
            .with_gas_limit(estimated_gas);

        let fee_ceiling: alloy::primitives::Uint<256, 4> =
            U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);

        Ok(Build {
            transaction: final_tx.clone(),
            metadata: TransactionMetadata {
                estimator,
                estimated_gas,
                cost: format_units(fee_ceiling, "ether")
                    .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?,
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
    ) -> Result<TransactionRequest, TransferBuilderError>;
}

/// Trait for checking if accounts have sufficient balance for transfers.
///
/// This trait defines a common interface for balance validation across different
/// asset types (ETH and ERC20 tokens). Implementations should verify that the
/// sender has enough balance to cover both the transfer amount and associated gas fees.
pub trait BalanceChecker {
    /// Checks if the sender has sufficient balance for the requested transfer.
    ///
    /// This method validates that the sender's account balance is sufficient to cover
    /// the requested transfer amount plus any associated transaction fees. The specific
    /// validation logic depends on the asset type being transferred.
    async fn check_balance(
        &self,
        _req: &TransferRequest,
        _ctx: &TransferContext,
        _estimated_gas: u64,
        _estimator: Eip1559Estimation,
    ) -> Result<(), TransferBuilderError> {
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
    ) -> Result<TransactionRequest, TransferBuilderError> {
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
    ) -> Result<(), TransferBuilderError> {
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
    ) -> Result<TransactionRequest, TransferBuilderError> {
        let value = parse_ether(&req.raw_amount)
            .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?;
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
    /// Checks ETH balance for transfers, accounting for both transfer amount and gas fees.
    ///
    /// This implementation validates that the sender has sufficient ETH to cover:
    /// 1. The requested transfer amount
    /// 2. Maximum estimated gas fees (gas_limit × max_fee_per_gas)
    ///
    /// If the balance is insufficient for gas fees alone, returns `InsufficientForGas`.
    /// If the balance covers gas fees but not the full transfer, returns `InsufficientEth`
    /// with the maximum amount that can be sent.
    async fn check_balance(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
        estimated_gas: u64,
        estimator: Eip1559Estimation,
    ) -> Result<(), TransferBuilderError> {
        let balance = ctx
            .provider
            .get_balance(req.sender)
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))?;
        let tx_value = parse_ether(&req.raw_amount)
            .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?;
        let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        let total_required = tx_value.saturating_add(fee_ceiling);

        if balance < total_required {
            if balance < total_required {
                let possible_send_amount = balance.saturating_sub(fee_ceiling);
                return Err(TransferBuilderError::InsufficientEther {
                    current_balance: format_units(balance, "ether")
                        .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?,
                    max_sendable: format_units(possible_send_amount, "ether")
                        .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?,
                });
            }
        }
        Ok(())
    }
}

pub struct TokenTransferBuilder;

impl TransferBuilder for TokenTransferBuilder {
    async fn build_transaction(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
    ) -> Result<TransactionRequest, TransferBuilderError> {
        let value = parse_units(&req.raw_amount, req.token.decimals)
            .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?
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
    /// Checks ERC20 token balance and ETH balance for gas fees.
    ///
    /// If the user has insufficient ETH for gas fees, it returns `InsufficientForGas`.
    /// If the user has insufficient tokens, it returns `InsufficientTokens`.
    async fn check_balance(
        &self,
        req: &TransferRequest,
        ctx: &TransferContext,
        estimated_gas: u64,
        estimator: Eip1559Estimation,
    ) -> Result<(), TransferBuilderError> {
        let contract =
            Erc20Contract::Erc20ContractInstance::new(req.token.address, ctx.provider.clone());
        let token_balance = contract
            .balanceOf(req.sender)
            .call()
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))?;
        let transfer_amount = parse_units(&req.raw_amount, req.token.decimals)
            .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?
            .get_absolute();
        if token_balance < transfer_amount {
            return Err(TransferBuilderError::InsufficientTokens);
        }
        let eth_balance = ctx
            .provider
            .get_balance(req.sender)
            .await
            .map_err(|e| TransferBuilderError::NodeQuery(e.to_string()))?;

        let fee_ceiling = U256::from(estimated_gas) * U256::from(estimator.max_fee_per_gas);
        if eth_balance < fee_ceiling {
            return Err(TransferBuilderError::InsufficientGas {
                current_balance: format_units(eth_balance, "ether")
                    .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?,
                estimated_fee: format_units(fee_ceiling, "ether")
                    .map_err(|e| TransferBuilderError::AmountParse(e.to_string()))?,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{
        constants::{ETH, USDT},
        new_provider_anvil,
    };
    use alloy::signers::k256::ecdsa::SigningKey;
    use alloy_provider::{PendingTransactionConfig, ext::AnvilApi};
    use alloy_signer_local::LocalSigner;

    fn get_estimator() -> Eip1559Estimation {
        Eip1559Estimation {
            max_fee_per_gas: 20000000000u128,         // 20 gwei
            max_priority_fee_per_gas: 2000000000u128, // 2 gwei
        }
    }
    const ESTIMATED_GAS: u64 = 21000u64;

    struct TestContext {
        provider: DynProvider,
        ctx: TransferContext,
        alice: LocalSigner<SigningKey>,
        bob: LocalSigner<SigningKey>,
        token: Token,
        builder: TxBuilder,
    }

    async fn test_context(alice_balance: &str) -> TestContext {
        let provider = new_provider_anvil();
        let ctx = TransferContext {
            provider: provider.clone(),
            nonce: 0,
        };
        let alice = LocalSigner::random();
        let bob = LocalSigner::random();
        if alice_balance != "0" {
            provider
                .anvil_set_balance(alice.address(), parse_ether(alice_balance).unwrap())
                .await
                .unwrap();
        }
        let builder = TxBuilder::new(provider.clone());
        TestContext {
            provider,
            ctx,
            alice,
            bob,
            token: USDT.clone(),
            builder,
        }
    }

    #[tokio::test]
    async fn test_send_eth() {
        let TestContext {
            provider,
            alice,
            bob,
            mut builder,
            ..
        } = test_context("1").await;
        let raw_amount = "0.01".to_string(); // 0.01 ETH
        builder
            .create_transfer(TransferRequest {
                token: ETH.clone(),
                raw_amount,
                sender: alice.address(),
                recipient: bob.address(),
            })
            .await
            .unwrap();
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
        let TestContext {
            token,
            provider,
            alice,
            bob,
            ..
        } = test_context("0.1").await;
        let mut builder = TxBuilder::new(provider.clone());
        let erc20_retriver = crate::eth::erc20_retriver::Erc20Retriever::new(provider.clone());
        let amount = "100.000000";
        provider
            .anvil_deal_erc20(
                alice.address(),
                token.address,
                parse_units(amount, token.decimals).unwrap().get_absolute(),
            )
            .await
            .unwrap();
        builder
            .create_transfer(TransferRequest {
                token: USDT.clone(),
                raw_amount: amount.to_string(),
                sender: alice.address(),
                recipient: bob.address(),
            })
            .await
            .unwrap();
        let tx_hash = builder.sign_and_send_tx(&alice).await.unwrap();
        let _tx_receipt = provider
            .watch_pending_transaction(PendingTransactionConfig::new(tx_hash))
            .await
            .unwrap();
        let balance = erc20_retriver
            .balances(bob.address(), vec![USDT.clone()])
            .await
            .unwrap();
        let token_balance = &balance[0];
        let recepien_balance = token_balance
            .token
            .get_balance(token_balance.balance)
            .to_string();
        assert_eq!(recepien_balance, amount);
    }

    #[tokio::test]
    async fn test_insufficient_token_balance_error() {
        let TestContext {
            token,
            provider,
            ctx,
            alice,
            bob,
            ..
        } = test_context("1").await;
        let deal_amount = "100";
        let transfer_amount = "1000";
        provider
            .anvil_deal_erc20(
                alice.address(),
                token.address,
                parse_units(deal_amount, token.decimals)
                    .unwrap()
                    .get_absolute(),
            )
            .await
            .unwrap();
        let req = TransferRequest {
            token: token.clone(),
            raw_amount: transfer_amount.to_string(),
            sender: alice.address(),
            recipient: bob.address(),
        };
        let result = TokenTransferBuilder
            .check_balance(&req, &ctx, ESTIMATED_GAS, get_estimator())
            .await;
        match result {
            Ok(_) => panic!(
                "Expected balance check to fail with InsufficientTokens error, but it succeeded"
            ),
            Err(e) => {
                assert_eq!(e, TransferBuilderError::InsufficientTokens);
            }
        }
    }

    #[tokio::test]
    async fn test_token_transfer_insufficient_eth_for_gas_fees() {
        let TestContext {
            provider,
            token,
            ctx,
            alice,
            bob,
            ..
        } = test_context("0").await;
        let token_builder = TokenTransferBuilder;
        let deal_amount = "1000";
        let transfer_amount = "100";
        provider
            .anvil_deal_erc20(
                alice.address(),
                token.address,
                parse_units(deal_amount, token.decimals)
                    .unwrap()
                    .get_absolute(),
            )
            .await
            .unwrap();
        let req = TransferRequest {
            token: token.clone(),
            raw_amount: transfer_amount.to_string(),
            sender: alice.address(),
            recipient: bob.address(),
        };
        let result = token_builder
            .check_balance(&req, &ctx, ESTIMATED_GAS, get_estimator())
            .await;
        match result {
            Ok(_) => panic!(
                "Expected balance check to fail with InsufficientForGas error due to zero ETH, but it succeeded"
            ),
            Err(e) => {
                match e {
                    TransferBuilderError::InsufficientGas {
                        current_balance,
                        estimated_fee,
                    } => {
                        // Verify we have zero balance but need fees
                        let balance_f64: f64 = current_balance.parse().unwrap();
                        let fee_f64: f64 = estimated_fee.parse().unwrap();
                        assert_eq!(balance_f64, 0.0, "Balance should be zero");
                        assert!(fee_f64 > 0.0, "Fee should be positive");
                    }
                    other => panic!("Expected InsufficientForGas error, but got: {:?}", other),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_insufficient_ether_balance_error() {
        let TestContext {
            ctx, alice, bob, ..
        } = test_context("0.5").await;
        let req = TransferRequest {
            token: ETH.clone(),
            raw_amount: "1.0".to_string(), // 1 ETH transfer
            sender: alice.address(),
            recipient: bob.address(),
        };
        let result = EtherTransferBuilder
            .check_balance(&req, &ctx, ESTIMATED_GAS, get_estimator())
            .await;
        match result {
            Ok(_) => panic!(
                "Expected balance check to fail with InsufficientEth error, but it succeeded"
            ),
            Err(e) => {
                match e {
                    TransferBuilderError::InsufficientEther { max_sendable, .. } => {
                        // Verify the max sendable amount is reasonable (should be less than 0.5 ETH minus gas fees)
                        let max_sendable_f64: f64 = max_sendable.parse().unwrap();
                        assert!(
                            max_sendable_f64 < 0.5,
                            "Max sendable should be less than 0.5 ETH"
                        );
                        assert!(max_sendable_f64 > 0.0, "Max sendable should be positive");
                    }
                    other => panic!("Expected InsufficientEth error, but got: {:?}", other),
                }
            }
        }
    }
}
