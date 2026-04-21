use serde::Deserialize;
use shush_rs::SecretBox;
use specta::Type;

use crate::{config::Config, persistence, utils, wallet::Wallet};

#[derive(Type, PartialEq, Deserialize)]
pub enum CreationFlow {
    Import,
    Generation,
}

pub struct WalletKeeper {
    pub repository: persistence::WalletRepository,
}

impl Default for WalletKeeper {
    fn default() -> Self {
        Self {
            repository: persistence::WalletRepository,
        }
    }
}

impl WalletKeeper {
    pub fn ls(&self) -> Result<Vec<String>, std::io::Error> {
        self.repository.ls()
    }

    pub fn create(
        &self,
        config: Config,
        mnemonic: &str,
        passphrase: &str,
        name: &str,
        flow: CreationFlow,
    ) -> Result<Wallet, String> {
        let name = if name.is_empty() {
            self.gen_wallet_name()?
        } else {
            name.to_string()
        };
        let birth_date = match flow {
            CreationFlow::Import => None,
            CreationFlow::Generation => Some(utils::now()),
        };
        let wallet = Wallet::new(
            config,
            name,
            mnemonic.to_string(),
            SecretBox::new(Box::new(passphrase.to_string())),
            birth_date,
        )?;
        self.repository.save(&wallet)?;
        Ok(wallet)
    }

    fn gen_wallet_name(&self) -> Result<String, String> {
        let existing_wallets = self.repository.ls().map_err(|e| e.to_string())?;

        let mut max_ordinal = 0;
        for wallet_name in existing_wallets {
            if !wallet_name.starts_with("Wallet_") {
                continue;
            }

            if let Some(ordinal_str) = wallet_name.strip_prefix("Wallet_")
                && let Ok(ordinal) = ordinal_str.parse::<u32>()
            {
                max_ordinal = max_ordinal.max(ordinal);
            }
        }

        let next_ordinal = max_ordinal + 1;
        Ok(format!("Wallet_{}", next_ordinal))
    }
}
