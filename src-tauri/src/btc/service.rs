use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::OnceCell;

use crate::{
    btc::{
        ActiveAccountDto,
        account::KeyDerivationPathLabelMap,
        key_derivation::{self, Change, ChildKeyDeriviationScheme},
        providers::electrum_adapter::ElectrumAdapter,
        utxo::Utxo,
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

    pub async fn list_utxos(&self) -> Result<Vec<UtxoDto>, String> {
        let mut sk = self.sk.lock().await;
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

    pub async fn sync_utxos(&self) -> Result<Vec<UtxoDto>, String> {
        let address_path_map = {
            let mut sk = self.sk.lock().await;
            let wallet = sk.wallet()?;
            let prk = wallet.btc_prk()?;
            wallet.btc.active_account()?.derive_address_path_map(&prk)
        };

        let received_utxos = self
            .transport()
            .await?
            .get_utxos(address_path_map)
            .await
            .map_err(|e| e.to_string())?;

        let mut sk = self.sk.lock().await;
        let wallet = sk.wallet()?;
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
}

impl Utxo {
    fn to_dto(&self, address_label_map: &KeyDerivationPathLabelMap) -> UtxoDto {
        UtxoDto {
            value: self.output.value.to_sat().to_string(),
            utxo_id: UtxoId {
                tx_id: self.tx_id.to_string(),
                vout: self.vout.to_string(),
            },
            deriv_path: self.derivation.to_string(),
            address_label: self.label(address_label_map),
        }
    }
}
