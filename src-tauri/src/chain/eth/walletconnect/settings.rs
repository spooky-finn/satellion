use serde::Serialize;
use specta::Type;

use super::request::METHODS;
use crate::{config::Config, eth::config::EthereumConfig};

#[derive(Serialize, Type)]
pub struct WalletConnectSettings {
    project_id: String,
    disabled_reason: Option<String>,
    chains: Vec<WalletConnectChain>,
    methods: Vec<String>,
}

#[derive(Clone, Serialize, Type)]
struct WalletConnectChain {
    id: u32,
    name: &'static str,
}

enum RpcEndpoint {
    Configured,
    Public(&'static str),
}

struct Network {
    chain: WalletConnectChain,
    rpc: RpcEndpoint,
}

impl Network {
    const fn new(id: u32, name: &'static str, rpc: RpcEndpoint) -> Self {
        Self {
            chain: WalletConnectChain { id, name },
            rpc,
        }
    }
}

const MAINNETS: [Network; 2] = [
    Network::new(1, "Ethereum", RpcEndpoint::Configured),
    Network::new(
        8453,
        "Base",
        RpcEndpoint::Public("https://mainnet.base.org"),
    ),
];

const TESTNETS: [Network; 2] = [
    Network::new(
        11155111,
        "Sepolia",
        RpcEndpoint::Public("https://ethereum-sepolia-rpc.publicnode.com"),
    ),
    Network::new(
        84532,
        "Base Sepolia",
        RpcEndpoint::Public("https://sepolia.base.org"),
    ),
];

#[derive(Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum WalletConnectConfigChange {
    Unchanged,
    ResetAndNotify,
    ResetOffline,
}

/// The settings that determine whether existing WalletConnect grants remain
/// valid.
#[derive(PartialEq, Eq)]
pub(crate) struct WalletConnectConfig<'a> {
    ethereum: &'a EthereumConfig,
    tor_enabled: bool,
}

impl<'a> From<&'a Config> for WalletConnectConfig<'a> {
    fn from(config: &'a Config) -> Self {
        Self {
            ethereum: &config.eth,
            tor_enabled: config.tor.enabled,
        }
    }
}

impl<'a> WalletConnectConfig<'a> {
    pub(crate) fn change_to(&self, next: &Self) -> WalletConnectConfigChange {
        if self == next {
            WalletConnectConfigChange::Unchanged
        } else if next.tor_enabled {
            WalletConnectConfigChange::ResetOffline
        } else {
            WalletConnectConfigChange::ResetAndNotify
        }
    }

    fn networks(&self) -> &'static [Network] {
        if self.ethereum.walletconnect_testnets {
            &TESTNETS
        } else {
            &MAINNETS
        }
    }

    fn disabled_reason(&self) -> Option<&'static str> {
        if self.tor_enabled {
            Some("WalletConnect is unavailable while Tor is enabled.")
        } else if self.ethereum.anvil {
            Some("WalletConnect is unavailable in Anvil mode.")
        } else {
            None
        }
    }

    pub(super) fn enabled(config: &'a Config) -> Result<Self, String> {
        let settings = Self::from(config);
        match settings.disabled_reason() {
            Some(reason) => Err(reason.into()),
            None => Ok(settings),
        }
    }

    pub(super) fn rpc_url(&self, chain: u32) -> Result<&'a str, String> {
        let network = self
            .networks()
            .iter()
            .find(|network| network.chain.id == chain)
            .ok_or("The requested network is disabled in WalletConnect settings")?;
        Ok(match network.rpc {
            RpcEndpoint::Configured => &self.ethereum.rpc_url,
            RpcEndpoint::Public(url) => url,
        })
    }

    pub(super) fn settings(&self) -> WalletConnectSettings {
        WalletConnectSettings {
            project_id: self.ethereum.walletconnect_project_id().to_string(),
            disabled_reason: self.disabled_reason().map(str::to_string),
            chains: self
                .networks()
                .iter()
                .map(|network| network.chain.clone())
                .collect(),
            methods: METHODS.iter().map(|method| method.to_string()).collect(),
        }
    }
}

pub(super) fn network_name(chain: u32) -> &'static str {
    MAINNETS
        .iter()
        .chain(&TESTNETS)
        .find(|network| network.chain.id == chain)
        .map_or("Unknown network", |network| network.chain.name)
}
