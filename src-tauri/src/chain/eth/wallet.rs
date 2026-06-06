use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};

use crate::{
    chain_trait::{AccountIndex, AssetTracker, ChainTrait, SecureKey},
    config::Config,
    eth::{
        constants::{self},
        dtos::EthereumUnlock,
        token::Token,
    },
};

pub struct EthereumWallet {
    pub config: Config,
    pub active_account: AccountIndex,
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
        Ok(EthereumUnlock {
            address: prk.expose().address().to_string(),
        })
    }
}

impl AssetTracker<Token> for EthereumWallet {
    fn track(&mut self, asset: Token) -> Result<(), String> {
        if self.tracked_tokens.contains(&asset) {
            return Err(format!("Token {} already tracked", asset.symbol));
        }
        self.tracked_tokens.push(asset);
        Ok(())
    }

    fn untrack(&mut self, token: Token) -> Result<(), String> {
        let len_before = self.tracked_tokens.len();
        self.tracked_tokens.retain(|t| *t != token);
        if self.tracked_tokens.len() == len_before {
            return Err(format!("Token address '{}' not tracked", token.address));
        }
        Ok(())
    }
}

impl EthereumWallet {
    pub fn build_prk(&self, mnemonic: &str, passphrase: &str) -> Result<Prk, String> {
        MnemonicBuilder::<English>::default()
            .phrase(mnemonic)
            .derivation_path("m/44'/60'/0'/0/0")
            .unwrap()
            .password(self.config.xprk_passphrase(passphrase))
            .build()
            .map_err(|e| format!("fail to derive eth signer: {}", e))
            .map(|signer| Prk { signer })
    }

    pub fn get_tracked_token(&self, token: Address) -> Option<&Token> {
        self.tracked_tokens
            .iter()
            .find(|each| each.address == token)
    }
}

impl EthereumWallet {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tracked_tokens: constants::default_tokens(),
            active_account: 0,
        }
    }
}

pub fn parse_addres(addres: &str) -> Result<Address, String> {
    Address::from_str(addres).map_err(|e| format!("Invalid Ethereum address: {}", e))
}
