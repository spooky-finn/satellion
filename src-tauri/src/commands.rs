use serde::Serialize;
use specta::{Type, specta};
use std::str::FromStr;
use zeroize::Zeroize;

use crate::{
    chain::btc,
    chain_trait::{AccountIndex, ChainTrait},
    config::{BlockChain, Config, constants},
    eth::{
        self, PriceFeed,
        constants::{BTC_USD_PRICE_FEED, ETH_USD_PRICE_FEED},
    },
    mnemonic,
    session::{SK, Session},
    wallet_keeper::{CreationFlow, WalletKeeper},
};

#[specta]
#[tauri::command]
pub async fn generate_mnemonic() -> Result<String, String> {
    mnemonic::new()
}

#[specta]
#[tauri::command]
pub async fn validate_address(
    chain: BlockChain,
    address: String,
    config: tauri::State<'_, Config>,
) -> Result<(), String> {
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
pub async fn create_wallet(
    mut mnemonic: String,
    mut passphrase: String,
    name: String,
    creation_type: CreationFlow,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    config: tauri::State<'_, Config>,
) -> Result<bool, String> {
    if creation_type == CreationFlow::Generation && passphrase.len() < constants::MIN_PASSPHRASE_LEN
    {
        return Err(format!(
            "Passphrase must contain at least {} characters",
            constants::MIN_PASSPHRASE_LEN
        ));
    }
    wallet_keeper.create(
        config.inner().clone(),
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
pub async fn get_wallets(
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<Vec<String>, String> {
    wallet_keeper.ls().map_err(|e| e.to_string())
}

#[specta]
#[tauri::command]
pub async fn switch_blockchain(chain: BlockChain, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    wallet.last_used_chain = chain;
    wallet.persist()?;
    Ok(())
}

#[specta]
#[tauri::command]
pub async fn add_account(
    chain: BlockChain,
    label: String,
    sk: tauri::State<'_, SK>,
) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account_index = match chain {
        BlockChain::Bitcoin => wallet.btc.create_account(label),
        BlockChain::Ethereum => todo!(),
    };
    wallet.persist()?;
    Ok(account_index)
}

#[specta]
#[tauri::command]
pub async fn switch_account(
    chain: BlockChain,
    account: AccountIndex,
    sk: tauri::State<'_, SK>,
) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    match chain {
        BlockChain::Bitcoin => wallet.btc.active_account = account,
        BlockChain::Ethereum => wallet.eth.active_account = account,
    }
    wallet.persist()?;
    Ok(())
}

#[derive(Type, Serialize)]
pub struct UnlockDto {
    ethereum: eth::wallet::EthereumUnlockDto,
    bitcoin: btc::service::BitcoinUnlockDto,
    last_used_chain: BlockChain,
}

#[derive(Type, Serialize)]
pub struct PriceFeedDto {
    btc_usd: u32,
    eth_usd: u32,
}

#[specta]
#[tauri::command]
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

#[specta]
#[tauri::command]
pub async fn unlock_wallet(
    wallet_name: String,
    mut passphrase: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
    config: tauri::State<'_, Config>,
) -> Result<UnlockDto, String> {
    let mut wallet =
        wallet_keeper
            .repository
            .load(config.inner().clone(), &wallet_name, &passphrase)?;

    let (eth_prk, btc_prk, last_used_chain) = {
        let eth_prk = wallet.eth_prk()?;
        let btc_prk = wallet.btc_prk()?;
        let last_used_chain = wallet.last_used_chain;
        (eth_prk, btc_prk, last_used_chain)
    };

    let (ethereum, bitcoin) = (
        wallet.eth.unlock((), &eth_prk)?,
        btc::service::unlock(&wallet.btc, &btc_prk)?,
    );

    let session = Session::new(wallet).with_inactivity_timeout(config.session_inactivity_timeout());
    sk.lock().await.set(session);

    passphrase.zeroize();
    Ok(UnlockDto {
        ethereum,
        bitcoin,
        last_used_chain,
    })
}

#[specta]
#[tauri::command]
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
    Ok(())
}

#[derive(Type, Serialize)]
pub struct UIConfig {
    eth_anvil: bool,
}

#[specta]
#[tauri::command]
pub async fn get_config(config: tauri::State<'_, Config>) -> Result<UIConfig, String> {
    Ok(UIConfig {
        eth_anvil: config.eth.anvil,
    })
}
