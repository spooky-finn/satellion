use std::sync::Arc;

use shush_rs::SecretBox;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    chain::btc,
    config::{Config, constants::BlockChain},
    eth, mnemonic,
    wallet_keeper::WalletKeeper,
};

#[derive(ZeroizeOnDrop, Zeroize)]
pub struct WalletSecret {
    pub(crate) mnemonic: String,
    pub(crate) passphrase: String,
}

pub type Secretik = Arc<SecretBox<WalletSecret>>;

impl WalletSecret {
    pub fn new(mnemonic: String, passphrase: String) -> Secretik {
        Arc::new(shush_rs::SecretBox::new(Box::new(Self {
            mnemonic,
            passphrase,
        })))
    }
}

pub struct Wallet {
    pub name: String,
    pub last_used_chain: BlockChain,
    pub birth_date: Option<u64>,
    pub version: u16,

    pub btc: btc::BitcoinWallet,
    pub eth: eth::EthereumWallet,

    pub keeper: WalletKeeper,
    pub config: Config,
}

impl Wallet {
    pub fn new(
        config: Config,
        name: String,
        mnemonic: String,
        passphrase: String,
        birth_date: Option<u64>,
    ) -> Result<Self, String> {
        mnemonic::validate(&mnemonic)?;
        let secret = WalletSecret::new(mnemonic, passphrase);
        Ok(Wallet {
            name,
            last_used_chain: BlockChain::Bitcoin,
            birth_date,
            version: 1,
            btc: btc::BitcoinWallet::new(config.clone(), Arc::clone(&secret)),
            eth: eth::EthereumWallet::new(config.clone(), secret),
            keeper: WalletKeeper::default(),
            config,
        })
    }

    pub fn persist(&self) -> Result<(), String> {
        self.keeper.repository.save(self)
    }

    pub fn mutate_btc<F, T>(&mut self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut btc::BitcoinWallet) -> Result<T, String>,
    {
        let res = f(&mut self.btc)?;
        self.persist()?;
        Ok(res)
    }

    pub fn mutate_eth<F>(&mut self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut eth::EthereumWallet) -> Result<(), String>,
    {
        f(&mut self.eth)?;
        self.persist()
    }
}
