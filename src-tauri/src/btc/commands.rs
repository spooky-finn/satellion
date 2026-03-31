use serde::{Deserialize, Serialize};
use specta::{Type, specta};

use crate::{
    btc::{
        ActiveAccountDto,
        key_derivation::{Change, ChildKeyDeriviationScheme},
        utxo::Utxo,
    },
    chain_trait::SecureKey,
    session::SK,
};

#[specta]
#[tauri::command]
pub async fn account_info(sk: tauri::State<'_, SK>) -> Result<ActiveAccountDto, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    wallet.btc.active_account_info(&prk)
}

#[specta]
#[tauri::command]
pub async fn derive_external_address(
    label: String,
    index: u32,
    sk: tauri::State<'_, SK>,
) -> Result<String, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    let path = wallet.btc.new_deriviation_path(Change::External, index)?;
    let derivation_scheme = ChildKeyDeriviationScheme { label, path };

    let child = derivation_scheme
        .path
        .derive(prk.expose())
        .map_err(|e| e.to_string())?;

    wallet
        .btc
        .get_mut_active_account()?
        .add_address(derivation_scheme);

    Ok(child.address.to_string())
}

#[specta]
#[tauri::command]
pub async fn unoccupied_deriviation_index(sk: tauri::State<'_, SK>) -> Result<u32, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    Ok(account.unoccupied_deriviation_index(Change::External))
}

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddressDto {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[specta]
#[tauri::command]
pub async fn get_external_addresess(
    sk: tauri::State<'_, SK>,
) -> Result<Vec<DerivedAddressDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let account = wallet.btc.active_account()?;
    let external_addresses: Vec<_> = account.get_external_addresess().collect();

    external_addresses
        .into_iter()
        .map(|scheme| {
            let child = scheme
                .path
                .derive(prk.expose())
                .map_err(|e| e.to_string())?;
            Ok(DerivedAddressDto {
                path: scheme.path.to_string(),
                label: scheme.label.clone(),
                address: child.address.to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

#[specta]
#[tauri::command]
pub async fn get_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let address_label_map = account.derivepath_label_map();

    let mut utxos: Vec<_> = account
        .utxos
        .values()
        .map(|utxo| utxo.to_dto(&address_label_map))
        .collect();

    utxos.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    const UTXO_DISPLAY_LIMIT: usize = 500;
    Ok(utxos.into_iter().take(UTXO_DISPLAY_LIMIT).collect())
}

#[derive(Type, Serialize, Deserialize)]
pub struct UtxoOutpoint {
    tx_id: String,
    vout: String,
}

#[derive(Type, Serialize)]
pub struct UtxoDto {
    pub utxo_id: UtxoOutpoint,
    pub value: String,
    pub deriv_path: String,
    pub address_label: Option<String>,
}

impl Utxo {
    fn to_dto(
        &self,
        address_label_map: &crate::btc::account::KeyDerivationPathLabelMap,
    ) -> UtxoDto {
        UtxoDto {
            value: self.output.value.to_sat().to_string(),
            utxo_id: UtxoOutpoint {
                tx_id: self.tx_id.to_string(),
                vout: self.vout.to_string(),
            },
            deriv_path: self.derivation.to_string(),
            address_label: self.label(address_label_map),
        }
    }
}

#[specta]
#[tauri::command]
pub async fn sync_utxos(sk: tauri::State<'_, SK>) -> Result<Vec<UtxoDto>, String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let address_path_map = wallet.btc.active_account()?.derive_address_path_map(&prk);

    let received_utxos = wallet
        .btc
        .server
        .get_utxos(address_path_map)
        .await
        .map_err(|e| e.to_string())?;

    let account = wallet.btc.get_mut_active_account()?;
    account.set_utxos(received_utxos);

    let address_label_map = account.derivepath_label_map();
    let mut result = account
        .utxos
        .values()
        .map(|utxo| utxo.to_dto(&address_label_map))
        .collect::<Vec<_>>();

    result.sort_by(|a, b| {
        b.value
            .parse::<u64>()
            .unwrap_or(0)
            .cmp(&a.value.parse::<u64>().unwrap_or(0))
    });

    wallet.persist()?;
    Ok(result)
}

#[derive(Type, Deserialize)]
pub struct BtcBuildTx {
    auto_utxo_selection: bool,
    utxos: Option<Vec<UtxoOutpoint>>,
    value: String,
    recipient: String,
}

#[specta]
#[tauri::command]
pub async fn build_tx(req: BtcBuildTx, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    Ok(())
}

#[derive(Type, Deserialize)]
pub struct BtcSendTx {}

#[specta]
#[tauri::command]
pub async fn send_tx(req: BtcSendTx, sk: tauri::State<'_, SK>) -> Result<(), String> {
    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;

    Ok(())
}
