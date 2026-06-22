use std::collections::HashMap;

use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid, hashes::Hash};
use serde::{Deserialize, Serialize};

use crate::{
    chain::btc::{
        account::{Account, KeyChain, UtxoSet},
        key_derivation::{KeyDerivationPath, LabeledKeyDerivationPath},
        providers::btc_node::select_btc_server,
        utxo::Utxo,
        wallet::BitcoinWallet,
    },
    chain_trait::AccountIndex,
    config::Config,
    wallet::Secretik,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtxoStored {
    txid: [u8; 32],
    vout: u32,
    value: u64,
    script_pubkey: Vec<u8>,
    derivation: KeyDerivationPath,
    height: u32,
}

impl From<&Utxo> for UtxoStored {
    fn from(u: &Utxo) -> Self {
        UtxoStored {
            txid: u.tx_id.to_byte_array(),
            vout: u.vout,
            value: u.output.value.to_sat(),
            script_pubkey: u.output.script_pubkey.to_bytes(),
            derivation: u.derivation.clone(),
            height: u.height,
        }
    }
}

impl From<UtxoStored> for Utxo {
    fn from(dto: UtxoStored) -> Self {
        Utxo {
            tx_id: Txid::from_byte_array(dto.txid),
            vout: dto.vout,
            output: TxOut {
                script_pubkey: ScriptBuf::from_bytes(dto.script_pubkey),
                value: Amount::from_sat(dto.value),
            },
            derivation: dto.derivation,
            height: dto.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AccountStored {
    name: String,
    index: AccountIndex,
    paths: Vec<LabeledKeyDerivationPath>,
    utxos: Vec<UtxoStored>,
}

impl From<&Account> for AccountStored {
    fn from(a: &Account) -> Self {
        AccountStored {
            name: a.name.clone(),
            index: a.index,
            paths: a.keychain.paths.clone(),
            utxos: a.utxo_set.entries.values().map(UtxoStored::from).collect(),
        }
    }
}

impl From<AccountStored> for Account {
    fn from(dto: AccountStored) -> Self {
        let entries: HashMap<OutPoint, Utxo> = dto
            .utxos
            .into_iter()
            .map(|u| {
                let utxo = Utxo::from(u);
                (utxo.outpoint(), utxo)
            })
            .collect();

        Account {
            name: dto.name,
            index: dto.index,
            keychain: KeyChain { paths: dto.paths },
            utxo_set: UtxoSet { entries },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletStored {
    active_account: AccountIndex,
    accounts: Vec<AccountStored>,
}

impl From<&BitcoinWallet> for WalletStored {
    fn from(w: &BitcoinWallet) -> Self {
        WalletStored {
            active_account: w.active_account,
            accounts: w.accounts.iter().map(AccountStored::from).collect(),
        }
    }
}

impl BitcoinWallet {
    pub fn from_dto(dto: WalletStored, config: Config, secret: Secretik) -> Self {
        let server = select_btc_server(&config);
        BitcoinWallet {
            secret,
            accounts: dto.accounts.into_iter().map(Account::from).collect(),
            active_account: dto.active_account,
            server,
            config,
            pending_tx: None,
        }
    }
}
