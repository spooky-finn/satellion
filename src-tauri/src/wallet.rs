use shush_rs::SecretBox;

use crate::{
    btc, config::constants::Chain, eth, mnemonic, utils::now, wallet_keeper::WalletKeeper,
};

pub struct Wallet {
    pub name: String,
    pub mnemonic: SecretBox<String>,
    pub passphrase: SecretBox<String>,
    pub last_used_chain: Chain,
    pub created_at: u64,
    pub version: u8,

    pub btc: btc::BitcoinWallet,
    pub eth: eth::EthereumWallet,

    pub keeper: WalletKeeper,
}

impl Wallet {
    pub fn new(
        name: String,
        mnemonic: String,
        passphrase: SecretBox<String>,
    ) -> Result<Self, String> {
        mnemonic::verify(&mnemonic)?;
        Ok(Wallet {
            name,
            mnemonic: SecretBox::new(Box::new(mnemonic)),
            passphrase,
            last_used_chain: Chain::Bitcoin,
            created_at: now(),
            version: 1,
            btc: btc::BitcoinWallet::default(),
            eth: eth::EthereumWallet::default(),
            keeper: WalletKeeper::new(),
        })
    }

    pub fn mutate_btc<F>(&mut self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut btc::BitcoinWallet) -> Result<(), String>,
    {
        f(&mut self.btc)?;
        self.persist()
    }

    pub fn mutate_eth<F>(&mut self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut eth::EthereumWallet) -> Result<(), String>,
    {
        f(&mut self.eth)?;
        self.persist()
    }

    pub fn persist(&self) -> Result<(), String> {
        self.keeper.save(self)
    }
}
