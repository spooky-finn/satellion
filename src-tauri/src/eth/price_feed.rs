use alloy::{
    primitives::{Address, U256, utils::format_units},
    sol,
};
use alloy_provider::DynProvider;

sol!(
    #[sol(rpc)]
    ChainlinkPriceFeed,
    "src/eth/abi/chainlink.json"
);

#[derive(Clone)]
pub struct PriceFeed {
    provider: DynProvider,
}

impl PriceFeed {
    pub fn new(provider: DynProvider) -> Self {
        Self { provider }
    }

    pub async fn get_price(&self, contract_address: Address) -> Result<String, String> {
        let contract = ChainlinkPriceFeed::ChainlinkPriceFeedInstance::new(
            contract_address,
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
