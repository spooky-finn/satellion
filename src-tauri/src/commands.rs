use serde::Serialize;
use shush_rs::ExposeSecret;
use specta::{Type, specta};
use tauri::AppHandle;
use zeroize::Zeroize;

use crate::{
    btc::{
        self,
        neutrino::{EventEmitter, NeutrinoStarter},
    },
    chain_trait::ChainTrait,
    config::{CONFIG, Chain, Config, constants},
    eth::{self, PriceFeed},
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
    let Session { wallet, .. } = sk.take_session()?;
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

#[specta]
#[tauri::command]
pub async fn unlock_wallet(
    app: AppHandle,
    wallet_name: String,
    passphrase: String,
    wallet_keeper: tauri::State<'_, WalletKeeper>,
    sk: tauri::State<'_, SK>,
    neutrino_starter: tauri::State<'_, NeutrinoStarter>,
    price_feed: tauri::State<'_, PriceFeed>,
) -> Result<UnlockMsg, String> {
    let mut wallet = wallet_keeper.load(&wallet_name, &passphrase)?;

    // Derive private keys for chains
    let eth_prk = wallet
        .eth
        .build_prk(&wallet.mnemonic.expose_secret(), &passphrase)?;
    let btc_prk = wallet
        .btc
        .build_prk(&wallet.mnemonic.expose_secret(), &passphrase)?;

    // Unlock both wallets in parallel using the ChainWallet trait
    let (ethereum, bitcoin) = tokio::try_join!(
        wallet.eth.unlock(price_feed.inner().clone(), &eth_prk),
        wallet.btc.unlock(price_feed.inner().clone(), &btc_prk)
    )?;

    let last_used_chain = wallet.last_used_chain;
    let btc_last_seen_heigh = wallet.btc.cfilter_scanner_height - 1;

    let event_emitter = EventEmitter::new(app);
    neutrino_starter
        .request_node_start(event_emitter, wallet_name, btc_last_seen_heigh)
        .await?;

    sk.lock()
        .await
        .start(Session::new(wallet, Config::session_exp_duration()));

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
    sk.lock().await.end();
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
