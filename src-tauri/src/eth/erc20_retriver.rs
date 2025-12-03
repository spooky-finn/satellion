use crate::eth::token::Token;
use alloy::{
    primitives::{Address, Uint},
    providers::Provider,
    sol,
};
use alloy_provider::DynProvider;
use futures;

sol!(
    #[sol(rpc)]
    Erc20Contract,
    "src/eth/abi/erc20.json"
);

pub fn new_contract_api(
    provider: DynProvider,
    contract: Address,
) -> Erc20Contract::Erc20ContractInstance<DynProvider> {
    Erc20Contract::Erc20ContractInstance::new(contract, provider)
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: Uint<256, 4>,
}

#[derive(Debug)]
pub struct Erc20Retriever {
    provider: DynProvider,
}

impl Erc20Retriever {
    pub fn new(provider: DynProvider) -> Self {
        Self { provider }
    }

    pub fn token_info(
        &self,
        token_address: Address,
    ) -> impl std::future::Future<Output = Result<Token, String>> + Send {
        async move {
            let erc20 = new_contract_api(self.provider.clone(), token_address);
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

    pub fn balances(
        &self,
        address: Address,
        tokens: Vec<Token>,
    ) -> impl std::future::Future<Output = Result<Vec<TokenBalance>, String>> + Send {
        async move {
            let balance_futures: Vec<_> = tokens
                .iter()
                .map(|token| {
                    let provider_clone = self.provider.root();
                    let contract = new_contract_api(self.provider.clone(), token.address);
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

            let results = futures::future::join_all(balance_futures).await;
            let balances: Result<Vec<_>, String> = results
                .into_iter()
                .zip(tokens)
                .map(|(result, token)| {
                    let token_symbol = token.symbol.clone();
                    result
                        .map_err(|e| format!("Failed to execute batch call: {}", e))?
                        .map(|balance| TokenBalance { token, balance })
                        .map_err(|e| format!("Failed to fetch balance for {}: {}", token_symbol, e))
                })
                .collect();

            Ok(balances?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{constants::DEFAULT_TOKENS, select_provider};
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_balance() {
        let address = Address::from_str("d8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap();
        let retriver = Erc20Retriever::new(select_provider());
        let tokens: Vec<Token> = DEFAULT_TOKENS.to_vec();
        let result = retriver.balances(address, tokens).await;
        assert!(result.is_ok(), "get_balances should not error");
        println!("res {:?}", result);

        if let Ok(balances) = result {
            for token_balance in &balances {
                assert!(!token_balance.token.symbol.is_empty());
                assert!(token_balance.token.decimals > 0);
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
        let retriver = Erc20Retriever::new(select_provider());
        // Test with USDC contract address
        let usdc_address = Address::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        let result = retriver.token_info(usdc_address).await;
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
