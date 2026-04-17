use bitcoin::Network;
use serde::Serialize;
use specta::Type;

use crate::{
    btc::{
        BitcoinWallet, Prk,
        account::Account,
        utxo::{OutPointDto, Utxo},
    },
    chain_trait::AccountIndex,
};

#[derive(Type, Serialize)]
pub struct UtxoDto {
    pub utxo_id: OutPointDto,
    pub value: String,
    pub deriv_path: String,
    pub address_label: Option<String>,
}

impl Utxo {
    pub fn to_dto(
        &self,
        address_label_map: &crate::btc::account::KeyDerivationPathLabelMap,
    ) -> UtxoDto {
        UtxoDto {
            value: self.output.value.to_sat().to_string(),
            utxo_id: self.outpoint(),
            deriv_path: self.derivation.to_string(),
            address_label: self.label(address_label_map),
        }
    }
}

#[derive(Serialize, specta::Type)]
pub struct AccountMetaDto {
    pub index: AccountIndex,
    pub name: String,
}

#[derive(Serialize, specta::Type)]
pub struct BitcoinUnlockDto {
    pub accounts: Vec<AccountMetaDto>,
    pub active_account: ActiveAccountDto,
}

pub fn unlock(wallet: &BitcoinWallet, prk: &Prk) -> Result<BitcoinUnlockDto, String> {
    let network = wallet.config.btc.network();
    let account = wallet.active_account()?;
    Ok(BitcoinUnlockDto {
        accounts: list_all_accounts(wallet),
        active_account: get_account_info(account, prk, network)?,
    })
}

pub fn list_all_accounts(wallet: &BitcoinWallet) -> Vec<AccountMetaDto> {
    wallet
        .accounts
        .iter()
        .map(|e| AccountMetaDto {
            index: e.index,
            name: e.name.clone(),
        })
        .collect()
}

#[derive(Serialize, specta::Type)]
pub struct ActiveAccountDto {
    pub index: u32,
    /** main external address to accept payments */
    pub address: String,
    pub total_balance: String,
    pub utxo: Vec<UtxoDto>,
}

pub fn get_account_info(
    account: &Account,
    prk: &Prk,
    network: Network,
) -> Result<ActiveAccountDto, String> {
    let (mainkey, _) = account.main_key(prk, network)?;
    let address_label_map = account.derive_path_label_map();

    let mut utxo: Vec<_> = account
        .utxos
        .values()
        .map(|utxo| utxo.to_dto(&address_label_map))
        .collect();
    utxo.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    Ok(ActiveAccountDto {
        index: account.index,
        address: mainkey.taproot_address.to_string(),
        total_balance: account.total_balance().to_string(),
        utxo,
    })
}
