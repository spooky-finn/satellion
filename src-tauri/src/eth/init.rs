use std::time::Duration;

use alloy::providers::RootProvider;
use alloy_provider::{DynProvider, Provider, ProviderBuilder};
use tauri::Url;

use crate::config::CONFIG;

pub fn select_provider() -> DynProvider {
    if CONFIG.ethereum.anvil {
        new_provider_anvil()
    } else {
        new_provider()
    }
}

pub fn new_provider() -> DynProvider {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
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
