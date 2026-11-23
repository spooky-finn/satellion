use alloy::primitives::Address;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::{
    chain_trait::SecureKey,
    config::Config,
    native_approval,
    session::{SK, Session},
    utils,
};

mod approval;
mod request;
mod review;
mod settings;
mod state;
mod transactions;
mod typed_data;

#[cfg(test)]
mod tests;

pub use request::WalletConnectRequest;
pub use review::WalletConnectReview;
pub(crate) use settings::WalletConnectConfig;
pub use settings::{WalletConnectConfigChange, WalletConnectSettings};
pub use state::WalletConnectGrant;
pub(crate) use state::WalletConnectState;

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_settings(
    config: State<'_, Mutex<Config>>,
) -> Result<WalletConnectSettings, String> {
    let config = config.lock().await;
    Ok(WalletConnectConfig::from(&*config).settings())
}

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_authorize(
    grant: WalletConnectGrant,
    config: State<'_, Mutex<Config>>,
    sk: State<'_, SK>,
) -> Result<(), String> {
    let config = config.lock().await;
    let settings = WalletConnectConfig::enabled(&config)?;
    grant.validate(&settings)?;

    let mut sk = sk.lock().await;
    let session = sk.unexpired_session()?;
    let address = active_address(session)?;
    session.walletconnect.authorize(grant, address)
}

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_disconnect(topic: String, sk: State<'_, SK>) -> Result<(), String> {
    if let Ok(session) = sk.lock().await.unexpired_session() {
        session.walletconnect.disconnect(&topic);
    }
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_prepare(
    request: WalletConnectRequest,
    config: State<'_, Mutex<Config>>,
    sk: State<'_, SK>,
) -> Result<WalletConnectReview, String> {
    request::prepare_request(&request, &config, &sk).await
}

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_reject(
    topic: String,
    id: String,
    sk: State<'_, SK>,
) -> Result<(), String> {
    if let Ok(session) = sk.lock().await.unexpired_session() {
        session.walletconnect.reject(&topic, &id);
    }
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn walletconnect_approve(
    topic: String,
    id: String,
    app: AppHandle,
    config: State<'_, Mutex<Config>>,
    sk: State<'_, SK>,
) -> Result<Option<String>, String> {
    approval::approve_request(&topic, &id, &config, &sk, async |approval| {
        let review = &approval.prepared.review;
        native_approval::confirm(
            &app,
            &review.title,
            &review.description(approval.address, approval.chain),
            "Approve",
            || approval.ensure_active(),
        )
        .await
    })
    .await
}

fn now() -> u32 {
    utils::now() as u32
}

fn active_address(session: &Session) -> Result<Address, String> {
    Ok(session.wallet.eth.prk()?.expose().address())
}
