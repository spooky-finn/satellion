use bip39::Language;
use bitcoin::bip32::{self, Xpriv};

use crate::{
    btc::{
        account::{Account, AccountIndex},
        key_derivation::{Change, KeyDerivationPath, LabeledDeriviationScheme},
    },
    chain_trait::{AssetTracker, ChainTrait, SecureKey},
    config::CONFIG,
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
}

impl Default for BitcoinWallet {
    fn default() -> BitcoinWallet {
        let active_account = 0;
        let account = Account::new(active_account, "main".to_string()).unwrap();
        BitcoinWallet {
            active_account,
            accounts: vec![account],
        }
    }
}

impl BitcoinWallet {
    pub fn build_prk(&self, mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
        let network = CONFIG.bitcoin.network();
        let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|e| e.to_string())?;
        let seed = mnemonic.to_seed(CONFIG.xprk_passphrase(passphrase));
        let xpriv = bip32::Xpriv::new_master(network, &seed).map_err(|e| e.to_string())?;
        Ok(Prk { xpriv })
    }

    pub fn active_account(&self) -> Result<&Account, String> {
        self.accounts
            .iter()
            .find(|each| each.index == self.active_account)
            .ok_or("account not found".to_string())
    }

    pub fn switch_account(&mut self, account: AccountIndex) {
        self.active_account = account;
    }

    pub fn new_deriviation_schema(
        &self,
        change: Change,
        index: u32,
    ) -> Result<KeyDerivationPath, String> {
        let account = self.active_account()?;
        let path = Account::new_deriviation_scheme_for_account(account.index, change, index);
        if !account.deriviation_schema_available(path.clone()) {
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

    pub fn active_account_info(&self, prk: &Prk) -> Result<ActiveAccountDto, String> {
        let account = self.active_account()?;
        let (_, btc_main_address) = Account::derive_child(
            prk.expose(),
            &self.new_deriviation_schema(Change::External, 0)?,
        )
        .map_err(|e| e.to_string())?;
        let active_account = ActiveAccountDto {
            address: btc_main_address.to_string(),
            total_balance: account.total_balance().to_string(),
        };
        Ok(active_account)
    }

    fn list_all_accounts(&self) -> Vec<AccountIdDto> {
        self.accounts
            .iter()
            .map(|e| AccountIdDto {
                index: e.index,
                name: e.name.clone(),
            })
            .collect()
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct ActiveAccountDto {
    /** main external address to accept payments */
    pub address: String,
    pub total_balance: String,
}

#[derive(serde::Serialize, specta::Type)]
pub struct AccountIdDto {
    pub index: AccountIndex,
    pub name: String,
}

#[derive(serde::Serialize, specta::Type)]
pub struct UnlockDto {
    pub accounts: Vec<AccountIdDto>,
    pub active_account: ActiveAccountDto,
}

pub struct UnlockCtx {}

impl ChainTrait for BitcoinWallet {
    type Prk = Prk;
    type AccountState = UnlockDto;
    type UnlockContext = ();

    fn unlock(
        &mut self,
        _: Self::UnlockContext,
        prk: &Self::Prk,
    ) -> Result<Self::AccountState, String> {
        Ok(UnlockDto {
            accounts: self.list_all_accounts(),
            active_account: self.active_account_info(&prk)?,
        })
    }
}

impl AssetTracker<LabeledDeriviationScheme> for BitcoinWallet {
    fn track(&mut self, address: LabeledDeriviationScheme) -> Result<(), String> {
        let account = self.get_mut_active_account()?;

        // Check if an address with the same purpose and index already exists
        if account.addresses.iter().any(|a| a.path == address.path) {
            return Err(format!(
                "Address with change {:?} and index {} already tracked",
                address.path.change, address.path.index
            ));
        }
        account.addresses.push(address);
        Ok(())
    }

    fn untrack(&mut self, address: LabeledDeriviationScheme) -> Result<(), String> {
        let account = self.get_mut_active_account()?;

        let len_before = account.addresses.len();
        account.addresses.retain(|a| a.path != address.path);
        if account.addresses.len() == len_before {
            return Err("Address not tracked".to_string());
        }
        Ok(())
    }
}

pub mod persistence {
    use serde::{Deserialize, Serialize};

    use crate::btc::{
        BitcoinWallet,
        account::{AccountIndex, persistence},
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
        pub fn deserialize(&self) -> Result<BitcoinWallet, String> {
            Ok(BitcoinWallet {
                accounts: self
                    .accounts
                    .iter()
                    .map(|each| each.deserialize().unwrap())
                    .collect(),
                active_account: self.active_account,
            })
        }
    }
}
