use shush_rs::SecretBox;

use crate::{persistence, wallet::Wallet};

pub struct WalletKeeper {
    repository: persistence::Repository,
}

impl WalletKeeper {
    pub fn new() -> Self {
        Self {
            repository: persistence::Repository,
        }
    }

    pub fn ls(&self) -> Result<Vec<String>, std::io::Error> {
        self.repository.ls()
    }

    pub fn create(&self, mnemonic: &str, passphrase: &str, name: &str) -> Result<(), String> {
        let name = if name.is_empty() {
            self.gen_wallet_name()?
        } else {
            name.to_string()
        };
        let wallet = Wallet::new(
            name,
            mnemonic.to_string(),
            SecretBox::new(Box::new(passphrase.to_string())),
        )?;
        self.repository.store(&wallet)?;
        Ok(())
    }

    pub fn load(&self, wallet_name: &str, passphrase: &str) -> Result<Wallet, String> {
        self.repository.load(
            wallet_name,
            SecretBox::new(Box::new(passphrase.to_string())),
        )
    }

    pub fn save(&self, wallet: &Wallet) -> Result<(), String> {
        self.repository.store(wallet)
    }

    pub fn delete(&self, wallet_name: &str) -> Result<(), std::io::Error> {
        self.repository.delete(wallet_name)
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
