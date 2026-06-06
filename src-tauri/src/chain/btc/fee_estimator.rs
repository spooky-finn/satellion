use crate::chain::btc::{
    config::BitcoinConfig,
    providers::{
        btc_node::BtcNode,
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
/// For the Electrum path: prefer mempool.space (current mempool state) and
/// fall back to the Electrum server estimate (conservative bitcoind value).
/// For the Esplora/Tor path: use the Esplora client directly (already proxied).
pub async fn estimate_fee_rate(server: &BtcNode, config: &BitcoinConfig) -> Result<f64, String> {
    let raw = match server {
        BtcNode::Electrum(electrum) => {
            if config.regtest {
                electrum_estimate(electrum).await?
            } else {
                match mempool_space_estimate().await {
                    Ok(rate) => rate,
                    Err(e) => {
                        tracing::warn!(
                            "mempool.space fee estimate failed, falling back to electrum: {e}"
                        );
                        electrum_estimate(electrum).await?
                    }
                }
            }
        }
        BtcNode::Esplora(esplora) => esplora_estimate(esplora).await?,
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

async fn esplora_estimate(esplora: &EsploraAdapter) -> Result<f64, String> {
    esplora
        .estimate_fee_sat_vb(STANDARD_FEE_TARGET_BLOCKS)
        .await
        .map_err(|e| e.to_string())
}
