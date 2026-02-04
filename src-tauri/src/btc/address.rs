use std::{fmt::Display, str::FromStr};

use bitcoin::bip32::DerivationPath;
pub use bitcoin::network::Network;

/// m / purpose' / coin_type' / account' / change / address_index
pub type DerivePathSlice = [u32; 5];

#[cfg(test)]
pub fn make_hardened(raw: DerivePathSlice) -> DerivePathSlice {
    [
        raw[0] + HARDENED,
        raw[1] + HARDENED,
        raw[2] + HARDENED,
        raw[3],
        raw[4],
    ]
}

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
    pub account: u32,
    pub network: Network,
    pub change: Change,
    pub index: u32,
}

const HARDENED: u32 = 0x80000000;

impl Display for DerivePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl DerivePath {
    pub fn to_string(&self) -> String {
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        format!(
            "m/{}'/{coin_type}'/{}'/{}/{}",
            self.purpose as u32, self.account, self.change as i32, self.index
        )
    }

    pub fn to_path(&self) -> Result<DerivationPath, String> {
        let str = self.to_string();
        DerivationPath::from_str(&str).map_err(|e| format!("fail to derive bip86_path: {e}"))
    }

    pub fn to_slice(&self) -> DerivePathSlice {
        let network = match self.network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        [
            HARDENED + self.purpose as u32,
            HARDENED + network,
            HARDENED + self.account,
            self.change as u32,
            self.index,
        ]
    }

    pub fn from_slice(v: DerivePathSlice) -> Result<Self, String> {
        let purpose = Purpose::try_from(
            v[0].checked_sub(HARDENED)
                .ok_or("purpose must be hardened")?,
        )?;
        let network = match v[1].checked_sub(HARDENED) {
            Some(0) => Network::Bitcoin,
            _ => Network::Regtest,
        };
        let account = v[2]
            .checked_sub(HARDENED)
            .ok_or("account must be hardened")?;
        let change = Change::try_from(v[3])?;
        let index = v[4];
        Ok(DerivePath {
            purpose,
            account,
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
    fn test_to_u32_vec() {
        let path = DerivePath {
            purpose: Purpose::Bip86,
            account: 0,
            network: Network::Bitcoin,
            change: Change::External,
            index: 0,
        };
        assert_eq!(path.to_slice(), make_hardened([86, 0, 0, 0, 0]));
    }

    #[test]
    fn test_from_u32_vec() {
        let vec = make_hardened([86, 0, 0, 0, 0]);
        let result = DerivePath::from_slice(vec);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.purpose, Purpose::Bip86);
        assert_eq!(path.network, Network::Bitcoin);
        assert_eq!(path.change, Change::External);
        assert_eq!(path.index, 0);
        assert_eq!(path.account, 0);
    }

    #[test]
    fn test_derive_path_roundtrip() {
        let original = DerivePath {
            purpose: Purpose::Bip86,
            account: 0,
            network: Network::Bitcoin,
            change: Change::Internal,
            index: 5,
        };
        let vec = original.to_slice();
        let parsed = DerivePath::from_slice(vec).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_derive_path_display() {
        let path = DerivePath {
            purpose: Purpose::Bip86,
            account: 0,
            network: Network::Bitcoin,
            change: Change::External,
            index: 0,
        };
        assert_eq!(path.to_string(), "m/86'/0'/0'/0/0");

        let path = DerivePath {
            purpose: Purpose::Bip86,
            account: 5,
            network: Network::Regtest,
            change: Change::Internal,
            index: 10,
        };
        assert_eq!(path.to_string(), "m/86'/1'/5'/1/10");
    }

    #[test]
    fn test_from_u32_vec_invalid_purpose() {
        let vec = [44, 0, 0, 0, 0]; // Invalid purpose
        assert!(DerivePath::from_slice(vec).is_err());
    }

    #[test]
    fn test_from_u32_vec_invalid_change() {
        let vec = [86, 0, 0, 2, 0]; // Invalid change
        assert!(DerivePath::from_slice(vec).is_err());
    }

    #[test]
    fn test_from_u32_vec_invalid_network() {
        let vec = [86, 99, 0, 0, 0]; // Invalid network
        assert!(DerivePath::from_slice(vec).is_err());
    }

    #[test]
    fn test_regtest_network() {
        let path = DerivePath {
            purpose: Purpose::Bip86,
            account: 0,
            network: Network::Regtest,
            change: Change::External,
            index: 10,
        };
        let vec = path.to_slice();
        assert_eq!(vec, vec);

        let parsed = DerivePath::from_slice(vec).unwrap();
        assert_eq!(parsed.network, Network::Regtest);
    }
}
