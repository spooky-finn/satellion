use serde::{Deserialize, Serialize};
use specta::Type;

pub const MIN_PASSPHRASE_LEN: usize = 4;

#[derive(Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockChain {
    Bitcoin = 0,
    Ethereum = 1,
}

impl From<u16> for BlockChain {
    fn from(value: u16) -> Self {
        match value {
            0 => BlockChain::Bitcoin,
            1 => BlockChain::Ethereum,
            _ => panic!("No default value for Chain. Invalid integer: {}", value),
        }
    }
}

impl From<BlockChain> for u16 {
    fn from(chain: BlockChain) -> Self {
        chain as u16
    }
}
