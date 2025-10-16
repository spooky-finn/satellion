use crate::app_state::AppState;

#[tauri::command]
pub async fn greet(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    let height = state.chain_height.read().await;
    height.ok_or_else(|| "Block height not available yet".to_string())
}

#[tauri::command]
pub async fn get_block_height(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    let height = state.chain_height.read().await;
    height.ok_or_else(|| "Block height not available yet".to_string())
}
