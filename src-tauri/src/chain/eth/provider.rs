use std::time::Duration;

use alloy::{
    consensus::TxEnvelope, primitives::B256, providers::RootProvider, rpc::client::RpcClient,
};
use alloy_provider::{DynProvider, Provider, ProviderBuilder};
use tauri::Url;

use crate::{config::Config, eth::config::EthereumConfig};

pub(super) async fn broadcast_signed(
    provider: &DynProvider,
    tx: TxEnvelope,
) -> Result<B256, String> {
    let expected = *tx.tx_hash();
    let pending = provider.send_tx_envelope(tx).await.map_err(|_| {
        format!("RPC did not confirm broadcast. Check transaction {expected} before retrying.")
    })?;
    if pending.tx_hash() != &expected {
        return Err(format!(
            "RPC returned an unexpected hash. Check transaction {expected} before retrying."
        ));
    }
    Ok(expected)
}

pub(super) async fn checked_http_provider(url: &str, chain: u32) -> Result<DynProvider, String> {
    let url = Url::parse(url).map_err(|_| "Invalid Ethereum RPC URL")?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|_| "Could not start the Ethereum RPC client")?;
    let provider = RootProvider::new(RpcClient::new_http_with_client(client, url)).erased();
    let actual_chain = provider
        .get_chain_id()
        .await
        .map_err(|_| "Could not verify the network of the Ethereum RPC")?;
    if actual_chain != u64::from(chain) {
        return Err(format!(
            "Ethereum RPC reports chain {actual_chain}; expected {chain}"
        ));
    }
    Ok(provider)
}

pub fn select_provider(config: Config) -> DynProvider {
    if config.eth.anvil {
        new_provider_anvil(config.eth)
    } else if config.tor.enabled {
        new_provider_tor(config.eth, &config.tor.socks5_proxy)
    } else {
        new_provider(config.eth)
    }
}

pub fn new_provider(config: EthereumConfig) -> DynProvider {
    let rpc_url = config.rpc_url.clone();
    RootProvider::new_http(Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum")).erased()
}

pub fn new_provider_tor(config: EthereumConfig, proxy_url: &str) -> DynProvider {
    let proxy = reqwest::Proxy::all(proxy_url).expect("invalid Tor proxy URL");
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("failed to build Tor HTTP client for Ethereum");
    let url = Url::parse(&config.rpc_url).expect("Invalid RPC URL for Ethereum");
    RootProvider::new(RpcClient::new_http_with_client(client, url)).erased()
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
