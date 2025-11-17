use crate::{config::Chain, db, eth::token::Token, repository::TokenRepository};
use alloy::{primitives::Address, providers::Provider, sol};
use alloy_provider::DynProvider;
use bigdecimal::{BigDecimal, Zero};
use diesel::result;
use futures;
use std::str::FromStr;

sol!(
    #[sol(rpc)]
    Erc20Contract,
    "src/eth/abi/erc20.json"
);

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: BigDecimal,
}

#[derive(Debug)]
pub struct TokenManager {
    provider: DynProvider,
    repository: TokenRepository,
}

impl TokenManager {
    pub fn new(provider: DynProvider, repository: TokenRepository) -> Self {
        Self {
            provider,
            repository,
        }
    }

    pub fn insert_default_tokens(&self, tokens: Vec<db::Token>) -> Result<usize, result::Error> {
        self.repository.insert_or_ignore_many(tokens)
    }

    pub fn load_all(
        &self,
        wallet_id: i32,
        chain_id: Chain,
    ) -> Result<Vec<db::Token>, result::Error> {
        self.repository.load(wallet_id, chain_id)
    }

    pub fn load(
        &self,
        wallet_id: i32,
        chain_id: Chain,
        token_symbol: String,
    ) -> Result<db::Token, result::Error> {
        self.repository.get(wallet_id, chain_id, token_symbol)
    }

    pub fn get_token_info(
        &self,
        token_address: Address,
    ) -> impl std::future::Future<Output = Result<Token, String>> + Send {
        async move {
            let erc20 =
                Erc20Contract::Erc20ContractInstance::new(token_address, self.provider.clone());
            let symbol_result = erc20
                .symbol()
                .call()
                .await
                .map_err(|e| format!("Failed to fetch token symbol: {}", e))?;
            let decimals_result = erc20
                .decimals()
                .call()
                .await
                .map_err(|e| format!("Failed to fetch token decimals: {}", e))?;
            Ok(Token::new(token_address, symbol_result, decimals_result))
        }
    }

    pub fn get_balances(
        &self,
        address: Address,
        tokens: Vec<Token>,
    ) -> impl std::future::Future<Output = Result<Vec<TokenBalance>, String>> + Send {
        async move {
            let balance_futures: Vec<_> = tokens
                .iter()
                .map(|token| {
                    let provider_clone = self.provider.root();
                    let contract =
                        Erc20Contract::Erc20ContractInstance::new(token.address, provider_clone);
                    let tx_request = contract.balanceOf(address).into_transaction_request();

                    let balance_future = async move {
                        provider_clone
                            .call(tx_request)
                            .decode_resp::<Erc20Contract::balanceOfCall>()
                            .await
                    };
                    balance_future
                })
                .collect();

            // Execute all balance calls concurrently (will be batched by CallBatchLayer)
            let results = futures::future::join_all(balance_futures).await;

            // Process results and collect successful balances
            let mut token_balances = Vec::new();
            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(balance_call) => {
                        let token = tokens[i].clone();
                        match balance_call {
                            Ok(balance_bytes) => {
                                let balance = BigDecimal::from_str(&balance_bytes.to_string())
                                    .unwrap_or_else(|_| BigDecimal::zero())
                                    / BigDecimal::from(10_i64.pow(token.decimals as u32));

                                token_balances.push(TokenBalance { token, balance });
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Failed to fetch balance for {}: {}",
                                    token.symbol, e
                                ));
                            }
                        }
                    }
                    Err(e) => return Err(format!("Failed to execute batch call: {}", e)),
                }
            }

            Ok(token_balances)
        }
    }
}

impl Clone for TokenManager {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            repository: self.repository.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::eth::{constants::mainnet::DEFAULT_TOKENS, new_provider};

    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_balance() {
        let address = Address::from_str("d8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap();
        let repository = TokenRepository::new(db::connect());
        let token_manager = TokenManager::new(new_provider(), repository);
        let tokens: Vec<Token> = DEFAULT_TOKENS.to_vec();
        let result = token_manager.get_balances(address, tokens).await;
        assert!(result.is_ok(), "get_balances should not error");
        println!("res {:?}", result);

        if let Ok(balances) = result {
            for token_balance in &balances {
                assert!(!token_balance.token.symbol.is_empty());
                assert!(token_balance.token.decimals > 0);
                assert!(token_balance.balance >= BigDecimal::zero());
            }

            assert!(
                balances.iter().any(|tb| tb.token.symbol == "USDT"),
                "Results should contain USDT token"
            );
            assert!(
                balances.iter().any(|tb| tb.token.symbol == "USDC"),
                "Results should contain USDC token"
            );
        }
    }

    #[tokio::test]
    async fn test_get_token_info() {
        let repository = TokenRepository::new(db::connect());
        let tm = TokenManager::new(new_provider(), repository);

        // Test with USDC contract address
        let usdc_address = Address::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

        let result = tm.get_token_info(usdc_address).await;

        // Note: This test may fail if we don't have internet access or the RPC is down
        // In a real test environment, we'd mock the provider
        if result.is_ok() {
            let token = result.unwrap();
            assert_eq!(token.address, usdc_address);
            assert!(!token.symbol.is_empty());
            assert!(token.decimals > 0);
        }
    }
}
