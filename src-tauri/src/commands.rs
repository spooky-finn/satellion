use crate::app_state::AppState;
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct SyncStatus {
    pub height: u32,
    pub sync_completed: bool,
}

#[tauri::command]
pub async fn chain_status(state: tauri::State<'_, Arc<AppState>>) -> Result<SyncStatus, String> {
    let height = state
        .chain_height
        .lock()
        .map_err(|_| "Failed to lock chain height".to_string())?;
    let sync_completed = state
        .sync_completed
        .lock()
        .map_err(|_| "Failed to lock sync completed".to_string())?;

    Ok(SyncStatus {
        height: *height,
        sync_completed: *sync_completed,
    })
}
