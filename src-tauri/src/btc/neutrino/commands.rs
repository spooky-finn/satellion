use std::sync::Arc;

use specta::specta;
use tauri::AppHandle;

use crate::{
    btc::neutrino::{EventEmitter, NeutrinoStarter, NodeStartArgs},
    session::SK,
};

#[specta]
#[tauri::command]
pub async fn btc_neutrino_start(
    app: AppHandle,
    sk: tauri::State<'_, SK>,
    neutrino_starter: tauri::State<'_, NeutrinoStarter>,
) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;

    let event_emitter = Arc::new(EventEmitter::new(app));
    neutrino_starter
        .request_node_start(
            NodeStartArgs {
                event_emitter,
                last_seen_height: wallet.btc.cfilter_scanner_height - 1,
            },
            wallet.name.clone(),
        )
        .await?;

    Ok(())
}
