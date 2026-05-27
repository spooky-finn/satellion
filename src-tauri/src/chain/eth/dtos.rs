use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    chain::eth::fee_estimator::FeeMode,
    config::BlockChain,
};

#[derive(Serialize, Type)]
pub struct EthereumUnlock {
    pub address: String,
}

#[derive(Serialize, Type)]
pub struct NetworkStatus {
    pub block_number: String,
    pub block_hash: String,
    pub base_fee_per_gas: Option<String>,
}

#[derive(Type, Serialize)]
pub struct TokenBalance {
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
    pub address: String,
}

#[derive(Type, Serialize)]
pub struct WalletBalance {
    pub wei: String,
    pub tokens: Vec<TokenBalance>,
}

#[derive(Type, Deserialize, Debug, PartialEq)]
pub struct TransferRequest {
    pub token_address: String,
    pub amount: String,
    pub recipient: String,
    pub fee_mode: FeeMode,
}

#[derive(Type, Serialize, Debug, PartialEq)]
pub struct TransferEstimation {
    pub estimated_gas: String,
    pub max_fee_per_gas: String,
    pub fee_ceiling: String,
    pub fee_in_usd: f64,
}

#[derive(Type, Serialize)]
pub struct TrackedTokenInfo {
    pub chain: BlockChain,
    pub symbol: String,
    pub decimals: i32,
}
