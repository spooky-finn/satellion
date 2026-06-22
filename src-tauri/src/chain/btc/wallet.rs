use bip39::Language;
use bitcoin::{
    Network,
    bip32::{self, Xpriv},
};
use shush_rs::ExposeSecret;

use crate::{
    chain::btc::{
        account::Account,
        dtos::{AccountSummary, BitcoinUnlock},
        key_derivation::{Change, KeyDerivationPath, Proposal},
        providers::btc_node::{BtcNode, select_btc_server},
        tx_builder::BuildTxResult,
    },
    chain_trait::{AccountIndex, SecureKey},
    config::Config,
    wallet::{Secretik, WalletSecret},
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
    pub(crate) secret: Secretik,
    pub active_account: AccountIndex,
    pub accounts: Vec<Account>,
    pub server: BtcNode,
    pub config: Config,
    pub pending_tx: Option<BuildTxResult>,
}

impl BitcoinWallet {
    pub fn new(config: Config, secret: Secretik) -> BitcoinWallet {
        let active_account = 0;
        let account = Account::new(config.btc.network(), active_account, "main".to_string());
        let server = select_btc_server(&config);
        BitcoinWallet {
            secret,
            config,
            active_account,
            accounts: vec![account],
            server,
            pending_tx: None,
        }
    }

    pub fn prk(&self) -> Result<Prk, String> {
        let secret = self.secret.expose_secret();
        let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, &secret.mnemonic)
            .map_err(|e| e.to_string())?;
        let seed = mnemonic.to_seed(self.config.xprk_passphrase(&secret.passphrase));
        let xpriv = bip32::Xpriv::new_master(self.config.btc.network(), &seed)
            .map_err(|e| e.to_string())?;
        Ok(Prk { xpriv })
    }

    pub fn unlock(&self) -> Result<BitcoinUnlock, String> {
        let network = self.config.btc.network();
        let account = self.active_account()?;
        let prk = self.prk()?;

        Ok(BitcoinUnlock {
            accounts: self.list_all_accounts(network)?,
            active_account: account.info(&prk, network)?,
        })
    }

    pub fn list_all_accounts(&self, network: Network) -> Result<Vec<AccountSummary>, String> {
        let prk = self.prk()?;
        self.accounts
            .iter()
            .map(|account| {
                let (main_key, _) = account.main_key(&prk, network)?;
                Ok(AccountSummary {
                    index: account.index,
                    name: account.name.clone(),
                    address: main_key.taproot_address.to_string(),
                })
            })
            .collect()
    }

    pub fn with_secret<T>(&self, f: impl FnOnce(&WalletSecret) -> T) -> T {
        f(&self.secret.expose_secret())
    }

    pub fn get_account(&self, index: u32) -> Result<&Account, String> {
        self.accounts
            .iter()
            .find(|each| each.index == index)
            .ok_or("account not found".to_string())
    }

    pub fn get_account_mut(&mut self, index: u32) -> Result<&mut Account, String> {
        self.accounts
            .iter_mut()
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

    pub fn rename_account(&mut self, account: AccountIndex, name: String) -> Result<(), String> {
        self.get_account_mut(account)?.name = name;
        Ok(())
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
        if account.keychain.contains_path(path.clone()) {
            return Err(format!("Derivation index {} already occupied", index));
        }
        Ok(path)
    }

    pub fn get_active_account_mut(&mut self) -> Result<&mut Account, String> {
        let active_index = self.active_account;
        self.accounts
            .iter_mut()
            .find(|each| each.index == active_index)
            .ok_or("account not found".to_string())
    }
}
