use bitcoin::Network;

use crate::chain::btc::{
    BitcoinWallet, Prk,
    account::Account,
    dtos::{AccountSummary, ActiveAccountView, BitcoinUnlock},
};

pub fn unlock(wallet: &BitcoinWallet, prk: &Prk) -> Result<BitcoinUnlock, String> {
    let network = wallet.config.btc.network();
    let account = wallet.active_account()?;
    Ok(BitcoinUnlock {
        accounts: list_all_accounts(wallet),
        active_account: get_account_info(account, prk, network)?,
    })
}

pub fn list_all_accounts(wallet: &BitcoinWallet) -> Vec<AccountSummary> {
    wallet
        .accounts
        .iter()
        .map(|e| AccountSummary {
            index: e.index,
            name: e.name.clone(),
        })
        .collect()
}

pub fn get_account_info(
    account: &Account,
    prk: &Prk,
    network: Network,
) -> Result<ActiveAccountView, String> {
    let (mainkey, _) = account.main_key(prk, network)?;
    let address_label_map = account.keychain.to_label_map();

    let mut utxo: Vec<_> = account
        .utxo_set
        .entries
        .values()
        .map(|utxo| utxo.to_view(&address_label_map))
        .collect();
    utxo.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    Ok(ActiveAccountView {
        index: account.index,
        address: mainkey.taproot_address.to_string(),
        total_balance: account.utxo_set.total_value().to_string(),
        utxo,
    })
}
