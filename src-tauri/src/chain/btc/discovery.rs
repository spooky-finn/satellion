//! Automated wallet discovery.
//!
//! Walks the derivation tree starting at `account 0`, scanning external
//! and internal chains with a sliding window bounded by an address gap limit.
//! Stops climbing to the next account once `account_gap_limit` consecutive
//! accounts come back empty. For every used address we then pull UTXOs.
//!
//! Designed to minimise network traffic:
//!   * Addresses are derived locally from the seed's `xpriv`.
//!   * Activity probing is done in one batched RPC per scan window
//!     (`scripthash.get_history` on Electrum, `/address/:addr` stats on
//!     Esplora).
//!   * Batch size is tuned per request from observed latency, clamped to a
//!     bounded range.
//!   * Per-account external and internal chains are scanned concurrently.
//!   * Empty windows are never rescanned: once a chain reaches its gap limit
//!     we move on.

use std::time::{Duration, Instant};

use bitcoin::{Address, Network, address::NetworkChecked, bip32::Xpriv};

use crate::{
    chain::btc::{
        BitcoinWallet,
        account::{Account, AddressPathMap},
        key_derivation::{Change, KeyDerivationPath, LabeledKeyDerivationPath, Proposal},
        providers::btc_node::BtcNode,
        utxo::Utxo,
    },
    chain_trait::AccountIndex,
};

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// BIP44 external chain gap limit (the canonical default is 20).
    pub external_gap_limit: u32,
    /// Internal (change) chain gap limit.
    pub internal_gap_limit: u32,
    /// Number of consecutive empty accounts before account scan stops.
    pub account_gap_limit: u32,
    /// Hard cap on accounts to prevent runaway scans.
    pub max_accounts: u32,
    /// Starting batch size for the sliding window.
    pub initial_batch_size: u32,
    pub min_batch_size: u32,
    pub max_batch_size: u32,
    /// Derivation schemes to discover, scanned in order.
    pub schemes: Vec<Proposal>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            external_gap_limit: 20,
            internal_gap_limit: 20,
            account_gap_limit: 3,
            max_accounts: 32,
            initial_batch_size: 50,
            min_batch_size: 10,
            max_batch_size: 200,
            schemes: vec![Proposal::Taproot, Proposal::SegWit],
        }
    }
}

#[derive(Debug, Default)]
struct ChainScanResult {
    used_paths: Vec<KeyDerivationPath>,
}

#[derive(Debug)]
pub struct DiscoveredAccount {
    pub scheme: Proposal,
    pub index: AccountIndex,
    pub external_paths: Vec<KeyDerivationPath>,
    pub internal_paths: Vec<KeyDerivationPath>,
    pub utxos: Vec<Utxo>,
}

#[derive(Debug, Default)]
pub struct DiscoveryReport {
    pub accounts: Vec<AccountIndex>,
    pub paths_added: usize,
    pub utxos_added: usize,
    pub total_value_sat: u64,
}

pub struct WalletDiscoverer<'a> {
    config: DiscoveryConfig,
    network: Network,
    server: &'a BtcNode,
    xpriv: &'a Xpriv,
}

impl<'a> WalletDiscoverer<'a> {
    pub fn new(server: &'a BtcNode, xpriv: &'a Xpriv, network: Network) -> Self {
        Self {
            config: DiscoveryConfig::default(),
            server,
            xpriv,
            network,
        }
    }

    pub fn with_config(mut self, config: DiscoveryConfig) -> Self {
        self.config = config;
        self
    }

    /// Run the full discovery procedure and return a list of accounts that
    /// have on-chain activity, populated with the used paths and their UTXOs.
    pub async fn discover(&self) -> Result<Vec<DiscoveredAccount>, String> {
        let mut discovered: Vec<DiscoveredAccount> = Vec::new();

        for scheme in self.config.schemes.iter().copied() {
            let mut empty_streak: u32 = 0;
            let mut account_idx: AccountIndex = 0;

            while account_idx < self.config.max_accounts {
                let (ext, int) = tokio::join!(
                    self.scan_chain(scheme, account_idx, Change::External),
                    self.scan_chain(scheme, account_idx, Change::Internal),
                );
                let ext = ext?;
                let int = int?;
                let used_total = ext.used_paths.len() + int.used_paths.len();

                if used_total == 0 {
                    empty_streak += 1;
                    tracing::debug!(
                        scheme = ?scheme,
                        account = account_idx,
                        streak = empty_streak,
                        "discovery: empty account"
                    );
                    if empty_streak >= self.config.account_gap_limit {
                        break;
                    }
                    account_idx += 1;
                    continue;
                }
                empty_streak = 0;

                let mut all_paths = Vec::with_capacity(ext.used_paths.len() + int.used_paths.len());
                all_paths.extend(ext.used_paths.iter().cloned());
                all_paths.extend(int.used_paths.iter().cloned());
                let path_map = self.materialize_address_map(scheme, &all_paths)?;
                let utxos = self.server.get_utxos(path_map).await?;

                tracing::info!(
                    scheme = ?scheme,
                    account = account_idx,
                    external = ext.used_paths.len(),
                    internal = int.used_paths.len(),
                    utxos = utxos.len(),
                    "discovery: active account"
                );

                discovered.push(DiscoveredAccount {
                    scheme,
                    index: account_idx,
                    external_paths: ext.used_paths,
                    internal_paths: int.used_paths,
                    utxos,
                });
                account_idx += 1;
            }
        }
        Ok(discovered)
    }

    async fn scan_chain(
        &self,
        scheme: Proposal,
        account: AccountIndex,
        change: Change,
    ) -> Result<ChainScanResult, String> {
        let gap_limit = match change {
            Change::External => self.config.external_gap_limit,
            Change::Internal => self.config.internal_gap_limit,
        };
        let mut batch_size = self.config.initial_batch_size;
        let mut next: u32 = 0;
        let mut max_used: Option<u32> = None;
        let mut used: Vec<KeyDerivationPath> = Vec::new();

        loop {
            // We stop once we've covered `gap_limit` consecutive empty
            // addresses past the last activity. Before any activity is found
            // that means covering [0, gap_limit).
            let target_end = match max_used {
                Some(m) => m.saturating_add(gap_limit).saturating_add(1),
                None => gap_limit,
            };
            if next >= target_end {
                break;
            }
            let end = next.saturating_add(batch_size);
            let indices: Vec<u32> = (next..end).collect();
            let (paths, addresses) = self.derive_window(scheme, account, change, &indices)?;

            let started = Instant::now();
            let activity = self.server.batch_has_activity(&addresses).await?;
            let elapsed = started.elapsed();
            batch_size = self.adapt_batch_size(batch_size, elapsed);

            if activity.len() != indices.len() {
                return Err(format!(
                    "provider returned {} activity flags for {} addresses",
                    activity.len(),
                    indices.len()
                ));
            }
            for (offset, has) in activity.into_iter().enumerate() {
                if has {
                    let i = indices[offset];
                    used.push(paths[offset].clone());
                    max_used = Some(max_used.map_or(i, |m| m.max(i)));
                }
            }
            next = end;
        }
        Ok(ChainScanResult { used_paths: used })
    }

    /// Adapt the batch size in response to observed round-trip latency. The
    /// goal is to keep windows below ~1s while still amortising connection
    /// overhead. Bounded by `[min_batch_size, max_batch_size]`.
    fn adapt_batch_size(&self, current: u32, elapsed: Duration) -> u32 {
        let ms = elapsed.as_millis() as u32;
        let next = if ms < 200 {
            current.saturating_mul(3) / 2
        } else if ms > 1000 {
            (current * 2).max(1) / 3
        } else {
            current
        };
        next.clamp(self.config.min_batch_size, self.config.max_batch_size)
    }

    fn derive_window(
        &self,
        scheme: Proposal,
        account: AccountIndex,
        change: Change,
        indices: &[u32],
    ) -> Result<(Vec<KeyDerivationPath>, Vec<Address<NetworkChecked>>), String> {
        let mut paths = Vec::with_capacity(indices.len());
        let mut addresses = Vec::with_capacity(indices.len());
        for &i in indices {
            let path = KeyDerivationPath::new(scheme, self.network, account, change, i);
            let child = path
                .derive(self.xpriv)
                .map_err(|e| format!("derive {}: {}", path, e))?;
            addresses.push(child.address_for(scheme).clone());
            paths.push(path);
        }
        Ok((paths, addresses))
    }

    fn materialize_address_map(
        &self,
        scheme: Proposal,
        paths: &[KeyDerivationPath],
    ) -> Result<AddressPathMap, String> {
        let mut map = AddressPathMap::new();
        for p in paths {
            let child = p
                .derive(self.xpriv)
                .map_err(|e| format!("derive {}: {}", p, e))?;
            map.insert(child.address_for(scheme).clone(), p.clone());
        }
        Ok(map)
    }
}

impl BitcoinWallet {
    /// Merge a discovery result into the in-memory wallet. Idempotent: paths
    /// and UTXOs already present are left untouched, accounts are inserted
    /// only when missing. Returns a summary of what was added.
    pub fn apply_discovery(&mut self, discovered: Vec<DiscoveredAccount>) -> DiscoveryReport {
        let network = self.config.btc.network();
        let mut report = DiscoveryReport::default();

        for da in discovered {
            if !self.accounts.iter().any(|a| a.index == da.index) {
                self.accounts.push(Account::new(
                    network,
                    da.index,
                    format!("Account {}", da.index),
                ));
            }
            if !report.accounts.contains(&da.index) {
                report.accounts.push(da.index);
            }

            let account = self
                .accounts
                .iter_mut()
                .find(|a| a.index == da.index)
                .expect("account inserted above");

            for p in da.external_paths.into_iter().chain(da.internal_paths) {
                if !account.keychain.contains_path(p.clone()) {
                    account.keychain.push(LabeledKeyDerivationPath {
                        label: String::new(),
                        path: p,
                    });
                    report.paths_added += 1;
                }
            }
            for u in da.utxos {
                let op = u.outpoint();

                if !account.utxo_set.entries.contains_key(&op) {
                    report.total_value_sat = report
                        .total_value_sat
                        .saturating_add(u.output.value.to_sat());
                    account.utxo_set.entries.insert(op, u);
                    report.utxos_added += 1;
                }
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapt_batch_size_grows_on_fast_responses() {
        let cfg = DiscoveryConfig::default();
        let server = make_dummy_server();
        let xpriv = dummy_xpriv();
        let d = WalletDiscoverer::new(&server, &xpriv, Network::Regtest).with_config(cfg.clone());

        let grown = d.adapt_batch_size(50, Duration::from_millis(50));
        assert!(grown > 50);
        assert!(grown <= cfg.max_batch_size);
    }

    #[test]
    fn adapt_batch_size_shrinks_on_slow_responses() {
        let cfg = DiscoveryConfig::default();
        let server = make_dummy_server();
        let xpriv = dummy_xpriv();
        let d = WalletDiscoverer::new(&server, &xpriv, Network::Regtest).with_config(cfg.clone());

        let shrunk = d.adapt_batch_size(50, Duration::from_millis(2000));
        assert!(shrunk < 50);
        assert!(shrunk >= cfg.min_batch_size);
    }

    #[test]
    fn adapt_batch_size_clamps_at_min() {
        let cfg = DiscoveryConfig::default();
        let server = make_dummy_server();
        let xpriv = dummy_xpriv();
        let d = WalletDiscoverer::new(&server, &xpriv, Network::Regtest).with_config(cfg.clone());

        let v = d.adapt_batch_size(cfg.min_batch_size, Duration::from_millis(5000));
        assert_eq!(v, cfg.min_batch_size);
    }

    fn make_dummy_server() -> BtcNode {
        use crate::chain::btc::{
            config::BitcoinConfig, providers::electrum_adapter::ElectrumAdapter,
        };
        BtcNode::Electrum(ElectrumAdapter::new(BitcoinConfig {
            regtest: true,
            electrum_server: None,
        }))
    }

    fn dummy_xpriv() -> Xpriv {
        Xpriv::new_master(Network::Regtest, &[0u8; 32]).unwrap()
    }
}
