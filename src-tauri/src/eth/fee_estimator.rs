use alloy::{
    providers::Provider,
    rpc::types::FeeHistory,
    transports::{RpcError, TransportErrorKind},
};
use alloy_provider::{DynProvider, utils::Eip1559Estimation};
use serde::Deserialize;
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
pub enum FeeMode {
    Minimal,
    Standard,
    Increased,
}

#[derive(Debug, Clone)]
pub struct FeeEstimations {
    pub minimal: Eip1559Estimation,
    pub standard: Eip1559Estimation,
    pub increased: Eip1559Estimation,
}

impl FeeEstimations {
    pub fn get(&self, fee_mode: FeeMode) -> &Eip1559Estimation {
        match fee_mode {
            FeeMode::Minimal => &self.minimal,
            FeeMode::Standard => &self.standard,
            FeeMode::Increased => &self.increased,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeePercentile {
    Minimal,
    Standard,
    Increased,
}

impl FeePercentile {
    /// Returns the percentile value for fee history queries
    pub const fn value(self) -> f64 {
        match self {
            FeePercentile::Minimal => 10.0,
            FeePercentile::Standard => 50.0,
            FeePercentile::Increased => 75.0,
        }
    }

    /// Returns the index in the fee history reward array
    pub const fn index(self) -> usize {
        match self {
            FeePercentile::Minimal => 0,
            FeePercentile::Standard => 1,
            FeePercentile::Increased => 2,
        }
    }
}

/// Introduction to Ethereum transaction fee related concepts
/// https://github.com/LearnWeb3DAO/What-is-Gas
pub struct FeeEstimator {
    provider: DynProvider,
}

impl FeeEstimator {
    const FEE_HISTORY_BLOCKS: u64 = 100;

    pub fn new(provider: DynProvider) -> Self {
        Self { provider }
    }

    pub async fn calc_fees(&self) -> Result<FeeEstimations, String> {
        let (fee_history, base_estimator) = tokio::join!(
            self.fetch_fee_history(),
            self.provider.estimate_eip1559_fees()
        );
        let fee_history = fee_history.map_err(|e| e.to_string())?;
        let base_estimator = base_estimator.map_err(|e| e.to_string())?;
        let base_fee = base_estimator.max_fee_per_gas - base_estimator.max_priority_fee_per_gas;

        let minimal_priority_fee =
            self.extract_percentile_fee(&fee_history, FeePercentile::Minimal)?;
        let standard_priority_fee =
            self.extract_percentile_fee(&fee_history, FeePercentile::Standard)?;
        let increased_priority_fee =
            self.extract_percentile_fee(&fee_history, FeePercentile::Increased)?;
        Ok(FeeEstimations {
            minimal: Eip1559Estimation {
                max_fee_per_gas: base_fee.saturating_add(minimal_priority_fee),
                max_priority_fee_per_gas: minimal_priority_fee,
            },
            standard: Eip1559Estimation {
                max_fee_per_gas: base_fee.saturating_add(standard_priority_fee),
                max_priority_fee_per_gas: standard_priority_fee,
            },
            increased: Eip1559Estimation {
                max_fee_per_gas: base_fee.saturating_add(increased_priority_fee),
                max_priority_fee_per_gas: increased_priority_fee,
            },
        })
    }

    /// Retrieves fee data from recent blocks to analyze market conditions.
    async fn fetch_fee_history(&self) -> Result<FeeHistory, RpcError<TransportErrorKind>> {
        self.provider
            .get_fee_history(
                Self::FEE_HISTORY_BLOCKS,
                Default::default(),
                &[
                    FeePercentile::Minimal.value(),
                    FeePercentile::Standard.value(),
                    FeePercentile::Increased.value(),
                ],
            )
            .await
    }

    fn extract_percentile_fee(
        &self,
        fee_history: &FeeHistory,
        percentile: FeePercentile,
    ) -> Result<u128, String> {
        let percentile_index = percentile.index();
        let mut fees_at_percentile: Vec<u128> = Vec::new();

        if let Some(reward_blocks) = &fee_history.reward {
            for rewards in reward_blocks.iter() {
                if rewards.len() > percentile_index {
                    let fee = rewards[percentile_index];
                    fees_at_percentile.push(fee);
                }
            }
        }

        if fees_at_percentile.is_empty() {
            return Err("No fee data available at the specified percentile".to_string());
        }

        fees_at_percentile.sort_unstable();
        let mid = fees_at_percentile.len() / 2;
        let median = fees_at_percentile[mid];
        Ok(std::cmp::max(median, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eth::{new_provider, new_provider_anvil};

    fn setup() -> (FeeEstimator, FeeHistory) {
        let fee_history = FeeHistory {
            base_fee_per_gas: vec![100, 110, 120, 130, 140],
            gas_used_ratio: vec![0.5, 0.6, 0.7, 0.8, 0.9],
            base_fee_per_blob_gas: vec![],
            blob_gas_used_ratio: vec![],
            reward: Some(vec![
                vec![10, 20, 30], // Block 1: minimal, standard, increased
                vec![15, 25, 35], // Block 2: minimal, standard, increased
                vec![12, 22, 32], // Block 3: minimal, standard, increased
                vec![18, 28, 38], // Block 4: minimal, standard, increased
                vec![9, 19, 29],  // Block 5: minimal, standard, increased
            ]),
            oldest_block: 100,
        };
        let estimator = FeeEstimator::new(new_provider_anvil());
        (estimator, fee_history)
    }

    #[test]
    fn test_extract_percentile_fee_minimal() {
        let (estimator, history) = setup();
        let minimal_fee = estimator
            .extract_percentile_fee(&history, FeePercentile::Minimal)
            .unwrap();
        // Extract all minimal fees: [10, 15, 12, 18, 9]
        // Sorted: [9, 10, 12, 15, 18]
        assert_eq!(minimal_fee, 12);
    }

    #[test]
    fn test_extract_percentile_fee_standard() {
        let (estimator, history) = setup();
        let standard_fee = estimator
            .extract_percentile_fee(&history, FeePercentile::Standard)
            .unwrap();
        // Extract all standard fees: [20, 25, 22, 28, 19]
        // Sorted: [19, 20, 22, 25, 28]
        assert_eq!(standard_fee, 22);
    }

    #[test]
    fn test_extract_percentile_fee_increased() {
        let (estimator, history) = setup();
        let increased_fee = estimator
            .extract_percentile_fee(&history, FeePercentile::Increased)
            .unwrap();
        // Extract all increased fees: [30, 35, 32, 38, 29]
        // Sorted: [29, 30, 32, 35, 38]
        // Median (index 2): 32
        assert_eq!(increased_fee, 32);
    }

    #[test]
    fn test_extract_percentile_fee_even_number_of_blocks() {
        let (estimator, mut history) = setup();
        // Modify to have even number of blocks (4 blocks)
        history.reward = Some(vec![
            vec![10, 20, 30],
            vec![15, 25, 35],
            vec![12, 22, 32],
            vec![18, 28, 38],
        ]);
        let minimal_fee = estimator
            .extract_percentile_fee(&history, FeePercentile::Minimal)
            .unwrap();
        // Extract all minimal fees: [10, 15, 12, 18]
        // Sorted: [10, 12, 15, 18]
        // For even length, len/2 = 2, so index 2: 15
        assert_eq!(minimal_fee, 15);
    }

    #[tokio::test]
    async fn test_calculate_all_fees() {
        let provider = new_provider();
        let estimator = FeeEstimator::new(provider.clone());
        let all_fees = estimator.calc_fees().await.unwrap();
        let minimal_fee = all_fees.get(FeeMode::Minimal);
        let standard_fee = all_fees.get(FeeMode::Standard);
        let increased_fee = all_fees.get(FeeMode::Increased);
        // Verify that fees are ordered correctly (minimal <= standard <= increased)
        assert!(minimal_fee.max_priority_fee_per_gas <= standard_fee.max_priority_fee_per_gas);
        assert!(standard_fee.max_priority_fee_per_gas <= increased_fee.max_priority_fee_per_gas);
    }
}
