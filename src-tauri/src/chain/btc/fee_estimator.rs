use crate::chain::btc::{
    config::BitcoinConfig,
    providers::{
        electrum_adapter::ElectrumAdapter,
        esplora_adapter::{EsploraAdapter, EsploraProvider},
    },
};

/// Target confirmation depth (in blocks) for the standard fee tier.
/// When a per-tx `FeeMode` picker (mirror of the Ethereum side) is added,
/// this becomes the target for the "Standard" variant.
pub const STANDARD_FEE_TARGET_BLOCKS: u16 = 2;

/// Floor for the sat/vB fee rate. Mainnet's default min relay fee is 1 sat/vB,
/// so anything below this wouldn't be relayed anyway.
pub const MIN_FEE_RATE_SAT_VB: f64 = 1.0;

/// Estimate the sat/vB fee rate for the standard confirmation target.
///
/// Electrum's `blockchain.estimatefee` proxies bitcoind's `estimatesmartfee`,
/// which is conservative and over-averaged — it routinely returns several×
/// the real next-block clearing rate. mempool.space's fee estimates reflect
/// current mempool state, so prefer it on mainnet and only fall back to the
/// connected Electrum server when the HTTP request fails (or on regtest,
/// where mempool.space isn't reachable).
pub async fn estimate_fee_rate(
    electrum: &ElectrumAdapter,
    config: &BitcoinConfig,
) -> Result<f64, String> {
    let raw = if config.regtest {
        electrum_estimate(electrum).await?
    } else {
        match mempool_space_estimate().await {
            Ok(rate) => rate,
            Err(e) => {
                tracing::warn!("mempool.space fee estimate failed, falling back to electrum: {e}");
                electrum_estimate(electrum).await?
            }
        }
    };
    let rate = if raw.is_finite() && raw > 0.0 {
        raw
    } else {
        MIN_FEE_RATE_SAT_VB
    };
    Ok(rate.max(MIN_FEE_RATE_SAT_VB))
}

async fn mempool_space_estimate() -> Result<f64, String> {
    let estimates = EsploraAdapter::new(EsploraProvider::MempoolSpace.main_net())
        .get_fee_estimates()
        .await
        .map_err(|e| e.to_string())?;
    // The endpoint returns block-target → sat/vB; pick our target, falling
    // back to the closest faster target if the exact one isn't present.
    (0..=STANDARD_FEE_TARGET_BLOCKS)
        .rev()
        .find_map(|target| estimates.get(&target).copied())
        .ok_or_else(|| "no fee estimate at target depth".to_string())
}

async fn electrum_estimate(electrum: &ElectrumAdapter) -> Result<f64, String> {
    electrum
        .estimate_fee(STANDARD_FEE_TARGET_BLOCKS as u32)
        .await
        .map_err(|e| format!("failed to estimate fee: {e}"))
}
