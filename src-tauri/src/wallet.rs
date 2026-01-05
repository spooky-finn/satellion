use shush_rs::SecretBox;

use crate::{btc, config::constants::Chain, eth, mnemonic, utils::now};

pub struct Wallet {
    pub name: String,
    pub mnemonic: SecretBox<String>,
    pub last_used_chain: Chain,
    pub created_at: u64,
    pub version: u8,

    pub btc: btc::BitcoinWallet,
    pub eth: eth::EthereumWallet,
}

impl Wallet {
    pub fn new(name: String, mnemonic: String) -> Result<Self, String> {
        mnemonic::verify(&mnemonic)?;
        Ok(Wallet {
            name,
            mnemonic: SecretBox::new(Box::new(mnemonic)),
            last_used_chain: Chain::Bitcoin,
            created_at: now(),
            version: 1,
            btc: btc::BitcoinWallet {
                derived_addresses: Vec::new(),
            },
            eth: eth::EthereumWallet {
                tracked_tokens: eth::constants::default_tokens(),
            },
        })
    }
}
