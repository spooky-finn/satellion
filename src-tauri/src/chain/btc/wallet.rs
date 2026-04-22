use bip39::Language;
use bitcoin::bip32::{self, Xpriv};

use crate::{
    chain::btc::{
        account::Account,
        key_derivation::{Change, KeyDerivationPath, Proposal},
        providers::electrum_adapter::ElectrumAdapter,
    },
    chain_trait::{AccountIndex, SecureKey},
    config::Config,
};

pub struct Prk {
    xpriv: Xpriv,
}

impl Drop for Prk {
    fn drop(&mut self) {
        self.xpriv.private_key.non_secure_erase();
    }
}

impl SecureKey for Prk {
    type Material = Xpriv;

    fn expose(&self) -> &Self::Material {
        &self.xpriv
    }
}

pub struct BitcoinWallet {
    pub active_account: AccountIndex,
    pub accounts: Vec<Account>,
    pub server: ElectrumAdapter,
    pub config: Config,
}

impl BitcoinWallet {
    pub fn new(config: Config) -> BitcoinWallet {
        let active_account = 0;
        let account = Account::new(config.btc.network(), active_account, "main".to_string());
        BitcoinWallet {
            config: config.clone(),
            active_account,
            accounts: vec![account],
            server: ElectrumAdapter::new(config.btc),
        }
    }

    pub fn build_prk(&self, mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
        let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|e| e.to_string())?;
        let seed = mnemonic.to_seed(self.config.xprk_passphrase(passphrase));
        let xpriv = bip32::Xpriv::new_master(self.config.btc.network(), &seed)
            .map_err(|e| e.to_string())?;
        Ok(Prk { xpriv })
    }

    pub fn get_account(&self, index: u32) -> Result<&Account, String> {
        self.accounts
            .iter()
            .find(|each| each.index == index)
            .ok_or("account not found".to_string())
    }

    pub fn active_account(&self) -> Result<&Account, String> {
        self.accounts
            .iter()
            .find(|each| each.index == self.active_account)
            .ok_or("account not found".to_string())
    }

    pub fn create_account(&mut self, label: String) -> AccountIndex {
        let next_index = self
            .accounts
            .iter()
            .map(|a| a.index)
            .max()
            .map(|i| i + 1)
            .unwrap_or(0);
        let account = Account::new(self.config.btc.network(), next_index, label);
        self.accounts.push(account);
        self.switch_account(next_index);
        next_index
    }

    pub fn switch_account(&mut self, account: AccountIndex) {
        self.active_account = account;
    }

    pub fn new_deriviation_path(
        &self,
        purpose: Proposal,
        change: Change,
        index: u32,
    ) -> Result<KeyDerivationPath, String> {
        let account = self.active_account()?;
        let path = KeyDerivationPath::new(
            purpose,
            self.config.btc.network(),
            account.index,
            change,
            index,
        );
        if !account.is_deriviation_path_available(path.clone()) {
            return Err(format!("Derivation index {} already occupied", index));
        }
        Ok(path)
    }

    pub fn get_mut_active_account(&mut self) -> Result<&mut Account, String> {
        let active_index = self.active_account;
        self.accounts
            .iter_mut()
            .find(|each| each.index == active_index)
            .ok_or("account not found".to_string())
    }
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    use crate::{
        chain::btc::{
            BitcoinWallet, account::persistence, providers::electrum_adapter::ElectrumAdapter,
        },
        chain_trait::AccountIndex,
        config::Config,
    };

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct WalletData {
        pub active_account: AccountIndex,
        pub accounts: Vec<persistence::AccountSnapshot>,
    }

    impl BitcoinWallet {
        pub fn serialize(&self) -> Result<WalletData, String> {
            Ok(WalletData {
                active_account: self.active_account,
                accounts: self
                    .accounts
                    .iter()
                    .map(|each| each.serialize().unwrap())
                    .collect(),
            })
        }
    }

    impl WalletData {
        pub fn deserialize(&self, config: Config) -> Result<BitcoinWallet, String> {
            Ok(BitcoinWallet {
                accounts: self
                    .accounts
                    .iter()
                    .map(|each| each.deserialize().unwrap())
                    .collect(),
                active_account: self.active_account,
                server: ElectrumAdapter::new(config.btc.clone()),
                config,
            })
        }
    }
}
