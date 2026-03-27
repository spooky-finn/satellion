use specta::specta;

use crate::btc::{
    key_derivation::Change,
    service::{DerivedAddressDto, UtxoDto, WalletService},
};

#[specta]
#[tauri::command]
pub async fn btc_account_info(
    service: tauri::State<'_, WalletService>,
) -> Result<crate::btc::ActiveAccountDto, String> {
    service.get_active_account_info().await
}

#[specta]
#[tauri::command]
pub async fn btc_derive_external_address(
    label: String,
    index: u32,
    service: tauri::State<'_, WalletService>,
) -> Result<String, String> {
    service.derive_external_address(label, index).await
}

#[specta]
#[tauri::command]
pub async fn btc_unoccupied_deriviation_index(
    service: tauri::State<'_, WalletService>,
) -> Result<u32, String> {
    service
        .get_unoccupied_deriviation_index(Change::External)
        .await
}

#[specta]
#[tauri::command]
pub async fn btc_get_external_addresess(
    service: tauri::State<'_, WalletService>,
) -> Result<Vec<DerivedAddressDto>, String> {
    service.list_external_addresses().await
}

#[specta]
#[tauri::command]
pub async fn btc_get_utxos(
    service: tauri::State<'_, WalletService>,
) -> Result<Vec<UtxoDto>, String> {
    service.list_utxos().await
}

#[specta]
#[tauri::command]
pub async fn btc_sync_utxos(
    service: tauri::State<'_, WalletService>,
) -> Result<Vec<UtxoDto>, String> {
    service.sync_utxos().await
}
