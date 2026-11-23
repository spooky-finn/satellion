use std::str::FromStr;

use serde::Serialize;
use shush_rs::{ExposeSecret, SecretBox};
use specta::{Type, specta};
use tokio::sync::Mutex;
use zeroize::Zeroize;

use crate::{
    chain::btc,
    chain_trait::AccountIndex,
    config::{BlockChain, Config, constants},
    eth::{
        self, PriceFeed,
        constants::{BTC_USD_PRICE_FEED, ETH_USD_PRICE_FEED},
        walletconnect::{WalletConnectConfig, WalletConnectConfigChange},
    },
    mnemonic,
    repository::{TxQuery, TxRecord, TxRepository},
    session::{SK, Session, SessionRevocation},
    system::biometric,
    wallet_keeper::{CreationFlow, WalletKeeper},
};

#[specta]
#[tauri::command]
#[tracing::instrument(name = "generate_mnemonic", skip_all, err)]
pub async fn generate_mnemonic() -> Result<String, String> {
    mnemonic::new()
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "validate_address", skip_all, err)]
pub async fn validate_address(
    chain: BlockChain,
    address: String,
    config: tauri::State<'_, Mutex<Config>>,
) -> Result<(), String> {
    let config = config.lock().await;
    match chain {
        BlockChain::Bitcoin => {
            bitcoin::Address::from_str(&address)
                .map_err(|e| format!("invalid address: {e}"))?
                .require_network(config.btc.network())
                .map_err(|e| format!("invalid address network: {e}"))?;
        }
        BlockChain::Ethereum => {
            alloy::primitives::Address::from_str(&address).map_err(|e| e.to_string())?;
        }
    };
    Ok(())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "create_wallet", skip_all, err)]
pub async fn create_wallet(
    mut mnemonic: String,
    mut passphrase: String,
    name: String,
    creation_type: CreationFlow,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    config: tauri::State<'_, Mutex<Config>>,
) -> Result<bool, String> {
    if creation_type == CreationFlow::Generation && passphrase.len() < constants::MIN_PASSPHRASE_LEN
    {
        return Err(format!(
            "Passphrase must contain at least {} characters",
            constants::MIN_PASSPHRASE_LEN
        ));
    }
    wallet_keeper.create(
        config.lock().await.clone(),
        &mnemonic,
        &passphrase,
        &name,
        creation_type,
    )?;

    mnemonic.zeroize();
    passphrase.zeroize();
    Ok(true)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "get_wallets", skip_all, err)]
pub async fn get_wallets(
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<Vec<String>, String> {
    wallet_keeper.ls().map_err(|e| e.to_string())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "switch_blockchain", skip_all, err)]
pub async fn switch_blockchain(chain: BlockChain, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    wallet.last_used_chain = chain;
    wallet.persist()?;
    Ok(())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "add_account", skip_all, err)]
pub async fn add_account(
    chain: BlockChain,
    label: String,
    sk: tauri::State<'_, SK>,
) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account_index = match chain {
        BlockChain::Bitcoin => wallet.btc.create_account(label),
        BlockChain::Ethereum => wallet.eth.create_account(label),
    };
    let result = wallet.persist();
    if chain == BlockChain::Ethereum {
        sk.clear_walletconnect();
    }
    result?;
    Ok(account_index)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "switch_account", skip_all, err)]
pub async fn switch_account(
    chain: BlockChain,
    account: AccountIndex,
    sk: tauri::State<'_, SK>,
) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let eth_account_changed = chain == BlockChain::Ethereum && wallet.eth.active_account != account;
    match chain {
        BlockChain::Bitcoin => wallet.btc.active_account = account,
        BlockChain::Ethereum => wallet.eth.switch_account(account)?,
    }
    let result = wallet.persist();
    if eth_account_changed {
        sk.clear_walletconnect();
    }
    result
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "rename_account", skip_all, err)]
pub async fn rename_account(
    chain: BlockChain,
    account: AccountIndex,
    name: String,
    sk: tauri::State<'_, SK>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Account name cannot be empty".to_string());
    }

    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    match chain {
        BlockChain::Bitcoin => wallet.btc.rename_account(account, name)?,
        BlockChain::Ethereum => wallet.eth.rename_account(account, name)?,
    }
    wallet.persist()?;
    Ok(())
}

#[derive(Type, Serialize)]
pub struct UnlockDto {
    ethereum: eth::dtos::EthereumUnlock,
    bitcoin: btc::dtos::BitcoinUnlock,
    last_used_chain: BlockChain,
}

#[derive(Type, Serialize)]
pub struct PriceFeedDto {
    btc_usd: u32,
    eth_usd: u32,
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "price_feed", skip_all, err)]
pub async fn price_feed(price_feed: tauri::State<'_, PriceFeed>) -> Result<PriceFeedDto, String> {
    let btc_usd = price_feed.get_price(BTC_USD_PRICE_FEED).await?;
    let eth_usd = price_feed.get_price(ETH_USD_PRICE_FEED).await?;

    let clean_price = |raw: String| -> Result<u32, String> {
        raw.parse::<f64>()
            .map(|f| f.round() as u32)
            .map_err(|_| format!("Failed to parse price: {}", raw))
    };

    Ok(PriceFeedDto {
        btc_usd: clean_price(btc_usd)?,
        eth_usd: clean_price(eth_usd)?,
    })
}

async fn do_unlock(
    wallet_name: &str,
    passphrase: &str,
    wallet_keeper: &WalletKeeper,
    sk: &SK,
    config: &Mutex<Config>,
    generation: u64,
) -> Result<UnlockDto, String> {
    let cfg = config.lock().await.clone();
    let mut wallet = wallet_keeper
        .repository
        .load(cfg.clone(), wallet_name, passphrase)?;

    let last_used_chain = wallet.last_used_chain;
    let (ethereum, bitcoin) = (wallet.eth.unlock()?, wallet.btc.unlock()?);

    let session = Session::new(wallet, cfg.session_inactivity_timeout());
    sk.lock().await.set_at(session, generation)?;

    Ok(UnlockDto {
        ethereum,
        bitcoin,
        last_used_chain,
    })
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "unlock_wallet", skip_all, err)]
pub async fn unlock_wallet(
    wallet_name: String,
    mut passphrase: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
    config: tauri::State<'_, Mutex<Config>>,
    revocation: tauri::State<'_, SessionRevocation>,
) -> Result<UnlockDto, String> {
    let result = do_unlock(
        &wallet_name,
        &passphrase,
        wallet_keeper.inner(),
        sk.inner(),
        config.inner(),
        revocation.generation(),
    )
    .await;
    passphrase.zeroize();
    result
}

#[specta]
#[tauri::command]
pub async fn lock_wallet(
    sk: tauri::State<'_, SK>,
    revocation: tauri::State<'_, SessionRevocation>,
) -> Result<(), String> {
    revocation.revoke();
    let sk = sk.inner().clone();
    tauri::async_runtime::spawn(async move {
        sk.lock().await.drop_revoked();
    });
    Ok(())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "unlock_wallet_with_biometric", skip_all, err)]
pub async fn unlock_wallet_with_biometric(
    wallet_name: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
    config: tauri::State<'_, Mutex<Config>>,
    revocation: tauri::State<'_, SessionRevocation>,
) -> Result<UnlockDto, String> {
    let generation = revocation.generation();
    let passphrase = biometric::prompt_unlock(&wallet_name).await?;
    do_unlock(
        &wallet_name,
        passphrase.expose_secret().as_str(),
        wallet_keeper.inner(),
        sk.inner(),
        config.inner(),
        generation,
    )
    .await
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "is_biometric_unlock_supported", skip_all)]
pub async fn is_biometric_unlock_supported() -> Result<bool, String> {
    Ok(biometric::is_supported())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "is_biometric_unlock_enabled", skip_all, err)]
pub async fn is_biometric_unlock_enabled(wallet_name: String) -> Result<bool, String> {
    biometric::is_enabled(&wallet_name).map_err(Into::into)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "enable_biometric_unlock", skip_all, err)]
pub async fn enable_biometric_unlock(sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let passphrase: biometric::Passphrase = SecretBox::new(Box::new(
        wallet.btc.with_secret(|secret| secret.passphrase.clone()),
    ));
    biometric::enable(&wallet.name, &passphrase).map_err(Into::into)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "disable_biometric_unlock", skip_all, err)]
pub async fn disable_biometric_unlock(wallet_name: String) -> Result<(), String> {
    biometric::disable(&wallet_name).map_err(Into::into)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "rename_wallet", skip_all, err)]
pub async fn rename_wallet(
    new_name: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let old_name = wallet.name.clone();
    let passphrase: biometric::Passphrase = SecretBox::new(Box::new(
        wallet.btc.with_secret(|secret| secret.passphrase.clone()),
    ));
    wallet_keeper.repository.rename(wallet, &new_name)?;
    let actual_name = wallet.name.clone();
    let _ = biometric::migrate(&old_name, &actual_name, &passphrase);
    Ok(actual_name)
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "forget_wallet", skip_all, err)]
pub async fn forget_wallet(
    wallet_name: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
) -> Result<(), String> {
    sk.lock().await.terminate();
    wallet_keeper
        .repository
        .delete(&wallet_name)
        .map_err(|e| e.to_string())?;
    biometric::forget(&wallet_name);
    Ok(())
}

#[specta]
#[tauri::command]
pub async fn get_config(config: tauri::State<'_, Mutex<Config>>) -> Result<Config, String> {
    Ok(config.lock().await.clone())
}

#[specta]
#[tauri::command]
pub async fn get_config_schema() -> Result<String, String> {
    serde_json::to_string(&schemars::schema_for!(Config)).map_err(|e| e.to_string())
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "set_config", skip_all, err)]
pub async fn set_config(
    input: Config,
    config: tauri::State<'_, Mutex<Config>>,
    sk: tauri::State<'_, SK>,
) -> Result<WalletConnectConfigChange, String> {
    let mut config = config.lock().await;
    input.save()?;
    let change = WalletConnectConfig::from(&*config).change_to(&WalletConnectConfig::from(&input));
    if change != WalletConnectConfigChange::Unchanged {
        sk.lock().await.clear_walletconnect();
    }
    *config = input;
    Ok(change)
}

#[specta]
#[tauri::command]
pub async fn mnemonic_wordlist() -> Result<&'static [&'static str], String> {
    Ok(mnemonic::word_list())
}

#[derive(Type, serde::Deserialize)]
pub struct ListTransactionsRequest {
    pub chain: BlockChain,
    pub account_index: i32,
    pub limit: Option<i32>,
}

#[specta]
#[tauri::command]
#[tracing::instrument(name = "list_transactions", skip_all, err)]
pub async fn list_transactions(
    req: ListTransactionsRequest,
    sk: tauri::State<'_, SK>,
    tx_repository: tauri::State<'_, TxRepository>,
) -> Result<Vec<TxRecord>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    tx_repository.list(&TxQuery {
        wallet_name: wallet.name.clone(),
        chain: req.chain,
        account_index: req.account_index,
        limit: req.limit.map(|l| l as i64),
    })
}
