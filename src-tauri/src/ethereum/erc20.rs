use crate::ethereum::{constants::mainnet::TOKENS, token::Token};
use alloy::{primitives::Address, providers::RootProvider, sol};
use bigdecimal::BigDecimal;
use futures::future;

sol!(
    #[sol(rpc)]
    Erc20Contract,
    "src/ethereum/abi/erc20.json"
);

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: BigDecimal,
}

async fn get_symbol(provider: RootProvider, contract_address: Address) -> Result<String, String> {
    let erc20 = Erc20Contract::Erc20ContractInstance::new(contract_address, provider);
    let symbol = erc20.symbol().call().await.map_err(|e| e.to_string())?;
    Ok(symbol)
}

pub async fn get_balances(
    provider: &RootProvider,
    address: Address,
) -> Result<Vec<TokenBalance>, String> {
    let balance_futures = TOKENS.iter().map(|token| {
        let token_clone = token.clone();
        let provider_clone = provider.clone();
        async move {
            let erc20 =
                Erc20Contract::Erc20ContractInstance::new(token_clone.address, provider_clone);
            let raw_balance = erc20.balanceOf(address).call().await.map_err(|e| {
                format!("Failed to fetch balance for {}: {}", token_clone.symbol, e)
            })?;
            let balance = token_clone.get_balance(raw_balance);
            Ok::<TokenBalance, String>(TokenBalance {
                token: token_clone,
                balance,
            })
        }
    });

    // Execute all queries in parallel using try_join_all
    // This will return an error if any of the queries fail
    let token_balances = future::try_join_all(balance_futures).await?;
    Ok(token_balances)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::address;

    use super::*;
    use crate::ethereum::{self};

    #[tokio::test]
    async fn test_get_symbol() {
        let provider = ethereum::client::new_client().unwrap();
        let contract_address = address!("dac17f958d2ee523a2206206994597c13d831ec7");
        let symbol = get_symbol(provider, contract_address).await.unwrap();
        assert_eq!(symbol, "USDT");
    }

    #[tokio::test]
    async fn test_get_balances() {
        let provider = ethereum::client::new_client().unwrap();
        let address = address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let balances = get_balances(&provider, address).await.unwrap();
        println!("{:?}", balances);
        assert_eq!(balances.len(), 5);
    }
}
