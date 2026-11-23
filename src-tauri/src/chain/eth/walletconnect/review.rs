use alloy::primitives::Address;
use serde::Serialize;
use specta::Type;

use super::settings::network_name;

#[derive(Clone, Default, Serialize, Type)]
pub struct WalletConnectReview {
    pub(super) title: String,
    pub(super) target: Option<String>,
    pub(super) value_eth: Option<String>,
    pub(super) fee_eth: Option<String>,
    pub(super) spender: Option<String>,
    pub(super) allowance: Option<String>,
    pub(super) details: Option<String>,
    pub(super) switch_chain: Option<u32>,
    pub(super) fields: Vec<(String, String)>,
}

impl WalletConnectReview {
    pub(super) fn description(&self, address: Address, chain: u32) -> String {
        let mut lines = vec![
            format!("WalletConnect account: {}", address.to_checksum(None)),
            format!("Network: {} ({chain})", network_name(chain)),
        ];
        let value_eth = self.value_eth.as_ref().map(|value| format!("{value} ETH"));
        let fee_eth = self.fee_eth.as_ref().map(|fee| format!("{fee} ETH"));
        let switch_chain = self
            .switch_chain
            .map(|chain| format!("{} ({chain})", network_name(chain)));
        for (label, value) in [
            ("Contract or recipient", self.target.as_deref()),
            ("ETH value", value_eth.as_deref()),
            ("Execution fee cap", fee_eth.as_deref()),
            ("Spender", self.spender.as_deref()),
            ("Allowance", self.allowance.as_deref()),
            ("Switch to", switch_chain.as_deref()),
        ] {
            if let Some(value) = value {
                lines.push(format!("{label}: {value}"));
            }
        }
        if self.fee_eth.is_some() && matches!(chain, 8453 | 84532) {
            lines.push("Base L1 data fees may apply separately.".into());
        }
        lines.extend(
            self.fields
                .iter()
                .map(|(label, value)| format!("{label}: {value}")),
        );
        if let Some(details) = &self.details {
            lines.push(details.clone());
        }
        lines.join("\n")
    }
}
