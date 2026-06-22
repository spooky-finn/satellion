use std::str::FromStr;

use crate::{
    chain_trait::{AccountIndex, AssetTracker, ChainTrait, SecureKey},
    config::Config,
    eth::{
        constants::{self},
        dtos::EthereumUnlock,
        token::Token,
    },
};
use alloy::primitives::Address;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};

pub struct EthereumWallet {
    pub config: Config,
    pub active_account: AccountIndex,
    pub accounts: Vec<Account>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub index: AccountIndex,
    pub name: String,
    pub tracked_tokens: Vec<Token>,
}

pub struct Prk {
    signer: PrivateKeySigner,
}

impl SecureKey for Prk {
    type Material = PrivateKeySigner;

    fn expose(&self) -> &Self::Material {
        &self.signer
    }
}

impl ChainTrait for EthereumWallet {
    type AccountState = EthereumUnlock;
    type Prk = Prk;
    type UnlockContext = ();

    fn unlock(
        &mut self,
        _: Self::UnlockContext,
        prk: &Self::Prk,
    ) -> Result<Self::AccountState, String> {
        let account = self.active_account()?;
        Ok(EthereumUnlock {
            accounts: self.account_summaries(),
            active_account: crate::eth::dtos::EthereumActiveAccountView {
                index: account.index,
                address: prk.expose().address().to_string(),
            },
        })
    }
}

impl AssetTracker<Token> for EthereumWallet {
    fn track(&mut self, asset: Token) -> Result<(), String> {
        let account = self.active_account_mut()?;
        if account.tracked_tokens.contains(&asset) {
            return Err(format!("Token {} already tracked", asset.symbol));
        }
        account.tracked_tokens.push(asset);
        Ok(())
    }

    fn untrack(&mut self, token: Token) -> Result<(), String> {
        let account = self.active_account_mut()?;
        let len_before = account.tracked_tokens.len();
        account.tracked_tokens.retain(|t| *t != token);
        if account.tracked_tokens.len() == len_before {
            return Err(format!("Token address '{}' not tracked", token.address));
        }
        Ok(())
    }
}

impl EthereumWallet {
    pub fn build_prk(&self, mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
        self.build_prk_for_account(mnemonic, passphrase, self.active_account)
    }

    pub fn build_prk_for_account(
        &self,
        mnemonic: &str,
        passphrase: &str,
        account: AccountIndex,
    ) -> Result<Prk, String> {
        self.get_account(account)?;
        MnemonicBuilder::<English>::default()
            .phrase(mnemonic)
            .derivation_path(&format!("m/44'/60'/{}'/0/0", account))
            .unwrap()
            .password(self.config.xprk_passphrase(passphrase))
            .build()
            .map_err(|e| format!("fail to derive eth signer: {}", e))
            .map(|signer| Prk { signer })
    }

    pub fn get_tracked_token(&self, token: Address) -> Option<&Token> {
        self.active_account()
            .ok()?
            .tracked_tokens
            .iter()
            .find(|each| each.address == token)
    }

    pub fn active_tracked_tokens(&self) -> Result<&[Token], String> {
        Ok(&self.active_account()?.tracked_tokens)
    }

    pub fn get_account(&self, index: AccountIndex) -> Result<&Account, String> {
        self.accounts
            .iter()
            .find(|account| account.index == index)
            .ok_or("account not found".to_string())
    }

    pub fn active_account(&self) -> Result<&Account, String> {
        self.get_account(self.active_account)
    }

    fn active_account_mut(&mut self) -> Result<&mut Account, String> {
        let index = self.active_account;
        self.accounts
            .iter_mut()
            .find(|account| account.index == index)
            .ok_or("account not found".to_string())
    }

    pub fn account_summaries(&self) -> Vec<crate::eth::dtos::EthereumAccountSummary> {
        self.accounts
            .iter()
            .map(|account| crate::eth::dtos::EthereumAccountSummary {
                index: account.index,
                name: account.name.clone(),
            })
            .collect()
    }

    pub fn create_account(&mut self, name: String) -> AccountIndex {
        let index = self
            .accounts
            .iter()
            .map(|account| account.index)
            .max()
            .map(|index| index + 1)
            .unwrap_or(0);
        self.accounts.push(Account {
            index,
            name,
            tracked_tokens: constants::default_tokens(),
        });
        self.active_account = index;
        index
    }

    pub fn switch_account(&mut self, index: AccountIndex) -> Result<(), String> {
        self.get_account(index)?;
        self.active_account = index;
        Ok(())
    }

    pub fn rename_account(&mut self, index: AccountIndex, name: String) -> Result<(), String> {
        self.accounts
            .iter_mut()
            .find(|account| account.index == index)
            .ok_or("account not found".to_string())?
            .name = name;
        Ok(())
    }
}

impl EthereumWallet {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            active_account: 0,
            accounts: vec![Account {
                index: 0,
                name: "main".to_string(),
                tracked_tokens: constants::default_tokens(),
            }],
        }
    }
}

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}

#[cfg(test)]
mod tests {
    use crate::{
        chain_trait::{AssetTracker, SecureKey},
        config::Config,
        eth::constants::ETH,
    };

    use super::EthereumWallet;

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn accounts_use_separate_bip44_derivation_paths() {
        let mut wallet = EthereumWallet::new(Config::default());
        let first = wallet.build_prk(MNEMONIC, "").unwrap();

        assert_eq!(wallet.create_account("frequent".to_string()), 1);
        let second = wallet.build_prk(MNEMONIC, "").unwrap();

        assert_ne!(first.expose().address(), second.expose().address());
    }

    #[test]
    fn account_lifecycle_keeps_a_valid_active_account() {
        let mut wallet = EthereumWallet::new(Config::default());
        let second = wallet.create_account("cold".to_string());

        assert_eq!(wallet.active_account, second);
        wallet
            .rename_account(second, "savings".to_string())
            .unwrap();
        wallet.switch_account(0).unwrap();

        assert_eq!(wallet.active_account().unwrap().name, "main");
        assert_eq!(wallet.get_account(second).unwrap().name, "savings");
        assert!(wallet.switch_account(99).is_err());
    }

    #[test]
    fn tracked_tokens_are_isolated_per_account() {
        let mut wallet = EthereumWallet::new(Config::default());
        let second = wallet.create_account("cold".to_string());

        wallet.track(ETH.clone()).unwrap();
        wallet.switch_account(0).unwrap();
        assert!(wallet.get_tracked_token(ETH.address).is_none());

        wallet.switch_account(second).unwrap();
        assert!(wallet.get_tracked_token(ETH.address).is_some());
    }
}
