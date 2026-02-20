use serde::Serialize;
use specta::{Type, specta};
use zeroize::Zeroize;

use crate::{
    btc::{
        self, UnlockCtx,
        neutrino::{NeutrinoStarter, NodeStartArgs},
    },
    chain_trait::ChainTrait,
    config::{CONFIG, Chain, constants},
    eth::{
        self, PriceFeed,
        constants::{BTC_USD_PRICE_FEED, ETH_USD_PRICE_FEED},
    },
    mnemonic,
    repository::ChainRepository,
    session::{SK, Session},
    wallet_keeper::{CreationFlow, WalletKeeper},
};

#[derive(Type, Serialize)]
pub struct ChainStatus {
    pub height: u32,
}

#[specta]
#[tauri::command]
pub async fn chain_status(
    chain_repository: tauri::State<'_, ChainRepository>,
) -> Result<ChainStatus, String> {
    let last_block = chain_repository
        .last_block()
        .map_err(|_| "Error getting last block height".to_string())?;
    Ok(ChainStatus {
        height: last_block.height as u32,
    })
}

#[specta]
#[tauri::command]
pub async fn generate_mnemonic() -> Result<String, String> {
    mnemonic::new()
}

#[specta]
#[tauri::command]
pub async fn create_wallet(
    mut mnemonic: String,
    mut passphrase: String,
    name: String,
    creation_type: CreationFlow,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<bool, String> {
    if creation_type == CreationFlow::Generation && passphrase.len() < constants::MIN_PASSPHRASE_LEN
    {
        return Err(format!(
            "Passphrase must contain at least {} characters",
            constants::MIN_PASSPHRASE_LEN
        ));
    }
    wallet_keeper.create(&mnemonic, &passphrase, &name, creation_type)?;

    mnemonic.zeroize();
    passphrase.zeroize();
    Ok(true)
}

#[specta]
#[tauri::command]
pub async fn list_wallets(
    wallet_keeper: tauri::State<'_, WalletKeeper>,
) -> Result<Vec<String>, String> {
    wallet_keeper.ls().map_err(|e| e.to_string())
}

#[specta]
#[tauri::command]
pub async fn chain_switch_event(chain: Chain, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    wallet.last_used_chain = chain;
    wallet.persist()?;
    Ok(())
}

#[derive(Type, Serialize)]
pub struct UnlockMsg {
    ethereum: eth::wallet::EthereumUnlock,
    bitcoin: btc::wallet::BitcoinUnlock,
    last_used_chain: Chain,
}

#[derive(Type, Serialize)]
pub struct PriceFeedMsg {
    btc_usd: u32,
    eth_usd: u32,
}

#[specta]
#[tauri::command]
pub async fn price_feed(price_feed: tauri::State<'_, PriceFeed>) -> Result<PriceFeedMsg, String> {
    let btc_usd = price_feed.get_price(BTC_USD_PRICE_FEED).await?;
    let eth_usd = price_feed.get_price(ETH_USD_PRICE_FEED).await?;

    let clean_price = |raw: String| -> Result<u32, String> {
        raw.parse::<f64>()
            .map(|f| f.round() as u32)
            .map_err(|_| format!("Failed to parse price: {}", raw))
    };

    Ok(PriceFeedMsg {
        btc_usd: clean_price(btc_usd)?,
        eth_usd: clean_price(eth_usd)?,
    })
}

#[specta]
#[tauri::command]
pub async fn unlock_wallet(
    // app: AppHandle,
    wallet_name: String,
    passphrase: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
    neutrino_starter: tauri::State<'_, NeutrinoStarter>,
) -> Result<UnlockMsg, String> {
    let mut wallet = wallet_keeper.load(&wallet_name, &passphrase)?;
    let eth_prk = wallet.eth_prk()?;
    let btc_prk = wallet.btc_prk()?;

    // Unlock both wallets in parallel using the ChainWallet trait
    let (ethereum, bitcoin) = tokio::try_join!(
        wallet.eth.unlock((), &eth_prk),
        wallet.btc.unlock(UnlockCtx {}, &btc_prk)
    )?;

    let last_used_chain = wallet.last_used_chain;

    // let event_emitter = EventEmitter::new(app);
    neutrino_starter
        .request_node_start(NodeStartArgs { birth_height: 0 }, wallet_name)
        .await?;

    sk.lock()
        .await
        .set(Session::new(wallet).with_inactivity_timeout(CONFIG.session_inactivity_timeout()));

    Ok(UnlockMsg {
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
pub async fn get_config() -> Result<UIConfig, String> {
    Ok(UIConfig {
        eth_anvil: CONFIG.ethereum.anvil,
    })
}
