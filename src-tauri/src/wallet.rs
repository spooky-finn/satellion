use zeroize::Zeroize;

use crate::{
    btc,
    config::{CONFIG, constants::Chain},
    eth, mnemonic,
    utils::now,
};

#[derive(Debug, PartialEq)]
pub struct Wallet {
    pub name: String,
    pub mnemonic: String,
    pub last_used_chain: Chain,
    pub created_at: u64,
    pub version: u8,

    pub btc: btc::wallet::WalletData,
    pub eth: eth::wallet::WalletData,
}

impl Wallet {
    pub fn new(name: String, mnemonic: String, passphrase: &str) -> Result<Self, String> {
        mnemonic::verify(&mnemonic)?;
        let bitcoin_data = btc::wallet::WalletData {
            xpriv: btc::wallet::create_private_key(CONFIG.bitcoin.network(), &mnemonic, passphrase)
                .map_err(|e| format!("Failed to create Bitcoin private key: {}", e))?,
            derived_addresses: Vec::new(),
        };
        let ethereum_data = eth::wallet::WalletData {
            signer: eth::wallet::create_private_key(&mnemonic, passphrase)
                .map_err(|e| format!("Failed to create Ethereum private key: {}", e))?,
            tracked_tokens: eth::constants::default_tokens(),
        };
        Ok(Wallet {
            name,
            mnemonic,
            last_used_chain: Chain::Bitcoin,
            created_at: now(),
            version: 1,
            btc: bitcoin_data,
            eth: ethereum_data,
        })
    }
}

impl Drop for Wallet {
    fn drop(&mut self) {
        self.mnemonic.zeroize();
    }
}
