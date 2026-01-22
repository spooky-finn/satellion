use std::{fmt::Display, str::FromStr};

use bitcoin::bip32::DerivationPath;
pub use bitcoin::network::Network;

#[derive(Debug, Clone, PartialEq)]
pub struct LabeledDerivationPath {
    pub label: String,
    pub derive_path: DerivePath,
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
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

impl TryFrom<u32> for Change {
    type Error = String;
    fn try_from(value: u32) -> Result<Change, String> {
        match value {
            0 => Ok(Change::External),
            1 => Ok(Change::Internal),
            _ => Err(format!("Invalid bitcoin address change: {}", value)),
        }
    }
}

impl From<Change> for u8 {
    fn from(chain: Change) -> Self {
        chain as u8
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum Purpose {
    Bip86 = 86,
}

impl TryFrom<u32> for Purpose {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            86 => Ok(Purpose::Bip86),
            v => Err(format!("invalid purpose {}", v)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DerivePath {
    pub purpose: Purpose,
    pub network: Network,
    pub change: Change,
    pub index: u32,
}

impl Display for DerivePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        let account = 0;
        let path = format!(
            "m/{}'/{coin_type}'/{account}'/{}/{}",
            self.purpose as u32, self.change as i32, self.index
        );
        f.write_str(&path)
    }
}

const HARDENED: u32 = 0x80000000;

impl DerivePath {
    pub fn to_path(&self) -> Result<DerivationPath, String> {
        DerivationPath::from_str(&self.to_string())
            .map_err(|e| format!("fail to derive bip86_path: {e}"))
    }

    pub fn from_str(path: &str) -> Result<Self, String> {
        let path_vec = DerivationPath::from_str(path)
            .map_err(|e| format!("fail to derive bip86_path: {e}"))?
            .to_u32_vec();

        let purpose = Purpose::try_from(
            path_vec
                .get(0)
                .copied()
                .ok_or("missing purpose component in derivation path")?
                - HARDENED,
        )?;

        let network = match path_vec.get(2).copied() {
            Some(HARDENED) => Network::Bitcoin,
            Some(x) if x == HARDENED + 1 => Network::Regtest,
            _ => return Err("invalid network component in derivation path".into()),
        };

        let change = Change::try_from(
            *path_vec
                .get(3)
                .ok_or("missing change component in bip86 path")?,
        )?;

        let index = path_vec
            .get(4)
            .copied()
            .ok_or("missing index component in bip86 path")?;

        Ok(DerivePath {
            purpose,
            network,
            change,
            index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_conversions() {
        assert_eq!(Change::try_from(0), Ok(Change::External));
        assert_eq!(Change::try_from(1), Ok(Change::Internal));
        assert!(Change::try_from(2).is_err());
    }

    #[test]
    fn test_purpose_conversion() {
        assert_eq!(Purpose::try_from(86), Ok(Purpose::Bip86));
        assert!(Purpose::try_from(44).is_err());
    }

    #[test]
    fn test_derive_path_display() {
        let path = DerivePath {
            purpose: Purpose::Bip86,
            network: Network::Bitcoin,
            change: Change::External,
            index: 0,
        };
        assert_eq!(path.to_string(), "m/86'/0'/0'/0/0");
    }

    #[test]
    fn test_derive_path_from_str() {
        let result = DerivePath::from_str("m/86'/0'/0'/0/0");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.purpose, Purpose::Bip86);
        assert_eq!(path.network, Network::Bitcoin);
        assert_eq!(path.change, Change::External);
        assert_eq!(path.index, 0);
    }

    #[test]
    fn test_derive_path_roundtrip() {
        let original = DerivePath {
            purpose: Purpose::Bip86,
            network: Network::Bitcoin,
            change: Change::Internal,
            index: 5,
        };
        let parsed = DerivePath::from_str(&original.to_string()).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_derive_path_invalid_input() {
        assert!(DerivePath::from_str("m/44'/0'/0'/0/0").is_err());
        assert!(DerivePath::from_str("m/86'/0'/0'/2/0").is_err());
        assert!(DerivePath::from_str("invalid").is_err());
    }
}
