use serde::{Deserialize, Serialize};
use specta::Type;

pub const MIN_PASSPHRASE_LEN: usize = 4;

#[derive(Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Bitcoin = 0,
    Ethereum = 1,
}

impl From<u16> for Chain {
    fn from(value: u16) -> Self {
        match value {
            0 => Chain::Bitcoin,
            1 => Chain::Ethereum,
            _ => panic!("No default value for Chain. Invalid integer: {}", value),
        }
    }
}

impl From<Chain> for u16 {
    fn from(chain: Chain) -> Self {
        chain as u16
    }
}
