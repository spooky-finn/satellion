use crate::{
    chain::btc::{
        account::AddressPathMap,
        providers::{electrum_adapter::ElectrumAdapter, esplora_adapter::EsploraAdapter},
        utxo::Utxo,
    },
    config::Config,
};

pub enum BtcNode {
    Electrum(ElectrumAdapter),
    Esplora(EsploraAdapter),
}

pub fn select_btc_server(config: &Config) -> BtcNode {
    if config.tor.enabled {
        BtcNode::Electrum(ElectrumAdapter::new_tor(&config.tor.socks5_proxy))
    } else {
        BtcNode::Electrum(ElectrumAdapter::new(config.btc.clone()))
    }
}

impl BtcNode {
    pub async fn get_utxos(&self, address_path_map: AddressPathMap) -> Result<Vec<Utxo>, String> {
        match self {
            BtcNode::Electrum(e) => e.get_utxos(address_path_map).await,
            BtcNode::Esplora(e) => e
                .get_wallet_utxos(address_path_map)
                .await
                .map_err(|e| e.to_string()),
        }
    }

    pub async fn broadcast_tx(&self, tx: &bitcoin::Transaction) -> Result<String, String> {
        match self {
            BtcNode::Electrum(e) => e.broadcast_tx(tx).await,
            BtcNode::Esplora(e) => e.broadcast_tx(tx).await.map_err(|e| e.to_string()),
        }
    }

    pub async fn estimate_fee(&self, blocks: u32) -> Result<f64, String> {
        match self {
            BtcNode::Electrum(e) => e.estimate_fee(blocks).await,
            BtcNode::Esplora(e) => e
                .estimate_fee_sat_vb(blocks as u16)
                .await
                .map_err(|e| e.to_string()),
        }
    }
}
