use crate::config::CONFIG;
use crate::eth::constants::USDT;
use crate::eth::wallet::EthereumUnlock;
use crate::repository::TokenRepository;
use crate::{db, eth::constants::DEFAULT_TOKENS};
use alloy::primitives::Address;
use alloy::primitives::utils::{parse_ether, parse_units};
use alloy::providers::RootProvider;
use alloy_provider::ext::AnvilApi;
use alloy_provider::{DynProvider, Provider, ProviderBuilder};
use std::str::FromStr;
use std::time::Duration;
use tauri::Url;

pub fn new_provider() -> DynProvider {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    if CONFIG.ethereum.anvil {
        return new_provider_anvil();
    }

    RootProvider::new_http(Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum")).erased()
}

pub fn new_provider_batched(provider: DynProvider) -> DynProvider {
    ProviderBuilder::new()
        .layer(alloy::providers::layers::CallBatchLayer::new().wait(Duration::from_millis(50)))
        .connect_provider(provider)
        .erased()
}

/// May panic if anvil is not installed
pub fn new_provider_anvil() -> DynProvider {
    let rpc_url = "https://reth-ethereum.ithaca.xyz/rpc";
    ProviderBuilder::new()
        .connect_anvil_with_config(|anvil| anvil.fork(rpc_url).path(CONFIG.ethereum.anvil_bin()))
        .erased()
}

pub async fn init_ethereum(
    provider: &DynProvider,
    token_repository: &TokenRepository,
    wallet_id: i32,
    unlock_data: &EthereumUnlock,
) -> Result<usize, String> {
    let tokens: Vec<db::Token> = DEFAULT_TOKENS
        .iter()
        .map(|token| db::Token {
            wallet_id,
            chain: 1, // Ethereum
            symbol: token.symbol.to_string(),
            address: token.address.as_slice().to_vec(),
            decimals: token.decimals as i32,
        })
        .collect();

    if CONFIG.ethereum.anvil {
        let wallet_address = Address::from_str(unlock_data.address.as_str())
            .map_err(|_| "failed to pasrse address")?;
        anvil_set_initial_balances(provider.clone(), wallet_address).await;
    }

    token_repository
        .insert_or_ignore_many(tokens)
        .map_err(|e| format!("failed to insert default ethereum tokens {}", e))
}

async fn anvil_set_initial_balances(provider: DynProvider, addr: Address) {
    let token = USDT.clone();
    provider
        .anvil_set_balance(addr, parse_ether("10").unwrap())
        .await
        .unwrap();

    provider
        .anvil_deal_erc20(
            addr,
            token.address,
            parse_units("9999999", token.decimals)
                .unwrap()
                .get_absolute(),
        )
        .await
        .unwrap();
}
