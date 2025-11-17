use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
pub enum Chain {
    Bitcoin = 0,
    Ethereum = 1,
}

impl From<i32> for Chain {
    fn from(value: i32) -> Self {
        match value {
            0 => Chain::Bitcoin,
            1 => Chain::Ethereum,
            _ => panic!("No default value for Chain. Invalid integer: {}", value),
        }
    }
}

impl From<Chain> for i32 {
    fn from(chain: Chain) -> Self {
        chain as i32
    }
}
