use crate::config::CONFIG;
use alloy::network::Ethereum;
use alloy::providers::RootProvider;
use tauri::Url;

pub fn new_client() -> Result<RootProvider, String> {
    let rpc_url = CONFIG.ethereum.rpc_url.clone();
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(&rpc_url).expect("Invalid RPC URL for Ethereum"),
    );
    Ok(provider)
}
