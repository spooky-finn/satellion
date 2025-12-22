use futures;

use alloy::{
    primitives::{Address, Uint},
    providers::Provider,
    sol,
};
use alloy_provider::DynProvider;

use crate::eth::token::Token;

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

    pub async fn token_info(&self, token_address: Address) -> Result<Token, String> {
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

    pub async fn balances(
        &self,
        address: Address,
        tokens: Vec<Token>,
    ) -> Result<Vec<TokenBalance>, String> {
        let balance_futures: Vec<_> = tokens
            .iter()
            .map(|token| {
                let provider_clone = self.provider.root();
                let contract = new_contract_api(self.provider.clone(), token.address);
                let tx_request = contract.balanceOf(address).into_transaction_request();
                async {
                    provider_clone
                        .call(tx_request)
                        .decode_resp::<Erc20Contract::balanceOfCall>()
                        .await
                }
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

        balances
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{
        constants::{USDC, USDT},
        select_provider,
    };
    use alloy::primitives::Address;
    use once_cell::sync::Lazy;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_balance() {
        let tokens: Lazy<Vec<Token>> = Lazy::<Vec<Token>>::new(|| vec![USDC.clone(), USDT.clone()]);
        let address = Address::from_str("d8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap();

        let retriver = Erc20Retriever::new(select_provider());
        let result = retriver.balances(address, tokens.to_vec()).await;
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
        let result = retriver.token_info(USDC.address).await;
        if result.is_ok() {
            let token = result.unwrap();
            assert_eq!(token.address, USDC.address);
            assert!(!token.symbol.is_empty());
            assert!(token.decimals > 0);
        }
    }
}
