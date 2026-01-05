use std::{fmt::Display, str::FromStr};

use bitcoin::bip32::DerivationPath;
pub use bitcoin::network::Network;

#[derive(Debug, Clone, PartialEq)]
pub struct BitcoinAddress {
    pub label: String,
    pub derive_path: DerivePath,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Change {
    /// External chain is used for addresses that are meant to be visible outside of the wallet (e.g. for receiving payments)
    External = 0,
    /// Internal chain is used for addresses which are not meant to be visible outside of the wallet and is used for return transaction change
    Internal = 1,
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::External => write!(f, "0"),
            Change::Internal => write!(f, "1"),
        }
    }
}

impl From<u8> for Change {
    fn from(value: u8) -> Self {
        match value {
            0 => Change::External,
            1 => Change::Internal,
            _ => panic!("Invalid bitcoin address change: {}", value),
        }
    }
}

impl From<Change> for u8 {
    fn from(chain: Change) -> Self {
        chain as u8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivePath {
    pub change: Change,
    pub index: u32,
}

impl Display for DerivePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.change, self.index)
    }
}

impl DerivePath {
    pub fn bip86_path(&self, network: Network) -> Result<DerivationPath, String> {
        let purpose = 86;
        let coin_type = match network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        let account = 0;
        let path = format!(
            "m/{purpose}'/{coin_type}'/{account}'/{}/{}",
            self.change as i32, self.index
        );
        DerivationPath::from_str(&path).map_err(|e| format!("fail to derive bip86_path: {e}"))
    }

    pub fn from_str(path: &str) -> Result<Self, String> {
        let path = DerivationPath::from_str(path)
            .map_err(|e| format!("fail to derive bip86_path: {e}"))?;
        let vec = path.to_u32_vec();
        let change: u8 = match vec.get(3).copied() {
            Some(0) => 0,
            Some(1) => 1,
            Some(v) => return Err(format!("invalid change value: {v}")),
            None => return Err("missing change component in bip86 path".into()),
        };
        let index = vec
            .get(4)
            .copied()
            .ok_or("missing index component in bip86 path")?;
        Ok(DerivePath {
            change: Change::from(change),
            index,
        })
    }
}
