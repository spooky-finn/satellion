use crate::config::CONFIG;
use crate::ethereum::constants::mainnet::ETH_USD_PRICE_FEED;
use alloy::network::Ethereum;
use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, U256};
use alloy::providers::RootProvider;
use alloy::sol;
use std::str::FromStr;
use tauri::Url;

sol!(
    #[sol(rpc)]
    ChainlinkPriceFeed,
    "src/ethereum/abi/chainlink.json"
);

pub struct PriceFeeder {
    provider: RootProvider<Ethereum>,
}

impl PriceFeeder {
    pub fn new() -> Result<Self, String> {
        let rpc_url = CONFIG.ethereum.rpc_url.clone();
        let provider = RootProvider::<Ethereum>::new_http(
            Url::parse(&rpc_url).map_err(|e| format!("Invalid RPC URL: {e}"))?,
        );
        Ok(Self { provider })
    }

    pub async fn get_eth_price(&self) -> Result<String, String> {
        let price_feed_address = Address::from_str(ETH_USD_PRICE_FEED)
            .map_err(|e| format!("Invalid price feed address: {e}"))?;

        let contract = ChainlinkPriceFeed::ChainlinkPriceFeedInstance::new(
            price_feed_address,
            self.provider.clone(),
        );

        let result = contract
            .latestRoundData()
            .call()
            .await
            .map_err(|e| format!("Failed to fetch price: {e}"))?;

        let price_u256 = if result.answer.is_negative() {
            return Err("Price is negative, which is unexpected".to_string());
        } else {
            // Convert signed int256 to U256 by taking the absolute value
            // Get the bytes representation and convert to U256
            let bytes = result.answer.to_be_bytes::<32>();
            U256::from_be_bytes(bytes)
        };
        // Format with 8 decimals (Chainlink standard)
        let price_str =
            format_units(price_u256, 8).map_err(|e| format!("Failed to format price: {e}"))?;
        Ok(price_str)
    }
}
