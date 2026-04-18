use std::time::Duration;

use alloy::providers::RootProvider;
use alloy_provider::{DynProvider, Provider, ProviderBuilder};
use tauri::Url;

use crate::eth::config::EthereumConfig;

pub fn select_provider(config: EthereumConfig) -> DynProvider {
    if config.anvil {
        new_provider_anvil(config)
    } else {
        new_provider(config)
    }
}

pub fn new_provider(config: EthereumConfig) -> DynProvider {
    let rpc_url = config.rpc_url.clone();
    RootProvider::new_http(Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum")).erased()
}

pub fn new_provider_batched(provider: DynProvider) -> DynProvider {
    ProviderBuilder::new()
        .layer(alloy::providers::layers::CallBatchLayer::new().wait(Duration::from_millis(50)))
        .connect_provider(provider)
        .erased()
}

/// May panic if anvil is not installed
pub fn new_provider_anvil(config: EthereumConfig) -> DynProvider {
    let rpc_url = "https://reth-ethereum.ithaca.xyz/rpc";
    ProviderBuilder::new()
        .connect_anvil_with_config(|anvil| anvil.fork(rpc_url).path(config.anvil_bin()))
        .erased()
}
