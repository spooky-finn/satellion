use alloy_provider::{Provider, RootProvider, network::Ethereum};
use tauri::Url;

const RPC_URL: &str = "https://ethereum-rpc.publicnode.com";

pub fn new_client() -> Result<impl Provider, String> {
    let provider = RootProvider::<Ethereum>::new_http(
        Url::parse(RPC_URL).expect("Invalid RPC URL for Ethereum"),
    );
    Ok(provider)
}
