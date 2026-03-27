use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::OnceCell;

use crate::{
    btc::{
        ActiveAccountDto,
        key_derivation::{self, Change, ChildKeyDeriviationScheme, DerivedAddress},
        providers::electrum_adapter::ElectrumAdapter,
    },
    chain_trait::SecureKey,
    session::SK,
};

#[derive(Type, Serialize, Deserialize)]
pub struct DerivedAddressDto {
    pub label: String,
    pub path: String,
    pub address: String,
}

#[derive(Type, Serialize, Deserialize)]
pub struct UtxoId {
    tx_id: String,
    vout: String,
}

#[derive(Type, Serialize)]
pub struct UtxoDto {
    pub utxo_id: UtxoId,
    pub value: String,
    pub deriv_path: String,
    pub address_label: Option<String>,
}

pub struct WalletService {
    sk: SK,
    connection: OnceCell<Arc<ElectrumAdapter>>,
}

impl WalletService {
    pub fn new(sk: SK) -> Self {
        Self {
            sk,
            connection: OnceCell::new(),
        }
    }

    async fn transport(&self) -> Result<Arc<ElectrumAdapter>, String> {
        self.connection
            .get_or_try_init(|| async { ElectrumAdapter::new().await.map(Arc::new) })
            .await
            .map_err(|e| e.to_string())
            .cloned()
    }

    pub async fn get_active_account_info(&self) -> Result<ActiveAccountDto, String> {
        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
        let prk = wallet.btc_prk()?;
        wallet.btc.active_account_info(&prk)
    }

    pub async fn derive_external_address(
        &self,
        label: String,
        index: u32,
    ) -> Result<String, String> {
        let mut sk = self.sk.lock().await;
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

    pub async fn get_unoccupied_deriviation_index(
        &self,
        change: key_derivation::Change,
    ) -> Result<u32, String> {
        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
        let account = wallet.btc.active_account()?;
        Ok(account.unoccupied_deriviation_index(change))
    }

    pub async fn list_external_addresses(&self) -> Result<Vec<DerivedAddressDto>, String> {
        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
        let prk = wallet.btc_prk()?;
        let account = wallet.btc.active_account()?;
        let external_addresses: Vec<_> = account.get_external_addresess().collect();

        Ok(external_addresses
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
            .collect::<Result<Vec<_>, String>>()?)
    }

    pub async fn list_utxos(&self) -> Result<Vec<UtxoDto>, String> {
        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
        let account = wallet.btc.active_account()?;
        let schema_label_map = account.derivepath_label_map();

        let mut utxos: Vec<_> = account
            .utxos
            .values()
            .map(|utxo| UtxoDto {
                value: utxo.output.value.to_sat().to_string(),
                utxo_id: UtxoId {
                    tx_id: utxo.tx_id.to_string(),
                    vout: utxo.vout.to_string(),
                },
                deriv_path: utxo.derivation.to_string(),
                address_label: utxo.label(&schema_label_map),
            })
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

    pub async fn sync_utxos(&self) -> Result<Vec<UtxoDto>, String> {
        let addresess = {
            let mut sk = self.sk.lock().await;
            let wallet = sk.wallet()?;
            let account = wallet.btc.active_account()?;

            let addresses: Vec<DerivedAddress> = account
                .addresses
                .iter()
                .filter(|a| a.path.change == Change::External)
                .map(|scheme| {
                    let prk = wallet.btc_prk()?;
                    let child = scheme
                        .path
                        .derive(prk.expose())
                        .map_err(|e| e.to_string())?;
                    Ok::<_, String>(DerivedAddress {
                        derive_path: scheme.path.clone(),
                        address: child.address,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            addresses
        };

        let utxos = self
            .transport()
            .await?
            .get_utxos(&addresess)
            .await
            .map_err(|e| e.to_string())?;

        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
        let result_utxos = {
            let account = wallet.btc.get_mut_active_account()?;
            account.utxos.clear();
            account.add_utxos(utxos);

            let schema_label_map = account.derivepath_label_map();
            account
                .utxos
                .values()
                .map(|utxo| UtxoDto {
                    value: utxo.output.value.to_sat().to_string(),
                    utxo_id: UtxoId {
                        tx_id: utxo.tx_id.to_string(),
                        vout: utxo.vout.to_string(),
                    },
                    deriv_path: utxo.derivation.to_string(),
                    address_label: utxo.label(&schema_label_map),
                })
                .collect::<Vec<_>>()
        };
        wallet.persist()?;

        let mut result_utxos = result_utxos;
        result_utxos.sort_by(|a, b| {
            b.value
                .parse::<u64>()
                .unwrap_or(0)
                .cmp(&a.value.parse::<u64>().unwrap_or(0))
        });

        const UTXO_DISPLAY_LIMIT: usize = 500;
        Ok(result_utxos.into_iter().take(UTXO_DISPLAY_LIMIT).collect())
    }
}
