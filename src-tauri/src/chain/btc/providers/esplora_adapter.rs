use std::collections::HashMap;

use bitcoin::{Address, TxOut};
use esplora_client::{AsyncClient, Builder, Error, Utxo, r#async::DefaultSleeper};
use futures::future::join_all;

use crate::chain::btc::{account::AddressPathMap, utxo::Utxo as WalletUtxo};

#[derive(Debug, Clone, Copy)]
pub enum EsploraProvider {
    MempoolSpace,
    BlockstreamInfo,
    MempoolEmzy,
}

impl EsploraProvider {
    pub const fn main_net(self) -> &'static str {
        match self {
            EsploraProvider::MempoolSpace => "https://mempool.space/api",
            EsploraProvider::BlockstreamInfo => "https://blockstream.info/api",
            EsploraProvider::MempoolEmzy => "https://mempool.emzy.de/api",
        }
    }

    pub const fn onion(self) -> &'static str {
        match self {
            EsploraProvider::MempoolSpace => {
                "http://mempoolhqx4isw62xs7abwphsq7ldayuidyx2v2oethdhhj6mlo2r6ad.onion/api"
            }
            EsploraProvider::BlockstreamInfo => {
                "http://explorerzydxu5ecjrkwceayqybizmpjjznk5izmitf2modhcusuqlid.onion/api"
            }
            EsploraProvider::MempoolEmzy => "http://mempool.emzy.de/api",
        }
    }
}

// Esplora API is a RESTful HTTP interface for querying Bitcoin blockchain data.
// It is developed by Blockstream and powers the public Blockstream Explorer.
// It allows clients (wallets, services, indexers) to fetch blockchain state
// without running a full node with a custom indexer.
pub struct EsploraAdapter {
    client: AsyncClient<DefaultSleeper>,
}

impl EsploraAdapter {
    pub fn new(base_url: &str) -> Self {
        let client = Builder::new(base_url)
            .build_async()
            .expect("fail to create EsploraClient");
        Self { client }
    }

    pub fn new_tor(proxy_url: &str) -> Self {
        let onion_url = EsploraProvider::MempoolSpace.onion();
        let client = Builder::new(onion_url)
            .proxy(proxy_url)
            .build_async()
            .expect("fail to create Tor EsploraClient");
        Self { client }
    }

    /// Get a map where the key is the confirmation target (in number of
    /// blocks) and the value is the estimated feerate (in sat/vB).
    pub async fn get_fee_estimates(&self) -> Result<HashMap<u16, f64>, Error> {
        self.client.get_fee_estimates().await
    }

    /// Fetches all UTXOs for a list of addresses.
    pub async fn get_utxos_by_addresses(
        &self,
        addresses: &[Address],
    ) -> Result<HashMap<Address, Vec<Utxo>>, Error> {
        let mut results = HashMap::new();

        let futures = addresses.iter().map(|addr| {
            let addr_clone = addr.clone();
            async move {
                let utxos = self.client.get_address_utxos(&addr_clone).await?;
                Ok::<(Address, Vec<Utxo>), Error>((addr_clone, utxos))
            }
        });

        let resolved = join_all(futures).await;

        for res in resolved {
            let (addr, utxos) = res?;
            results.insert(addr, utxos);
        }

        Ok(results)
    }

    /// Fetch UTXOs for all addresses in the path map and return them as wallet
    /// UTXOs.
    pub async fn get_wallet_utxos(
        &self,
        address_path_map: AddressPathMap,
    ) -> Result<Vec<WalletUtxo>, Error> {
        let addresses: Vec<Address> = address_path_map.keys().cloned().collect();
        let utxo_map = self.get_utxos_by_addresses(&addresses).await?;

        let result = utxo_map
            .into_iter()
            .flat_map(|(address, utxos)| {
                let path = address_path_map.get(&address).cloned();
                utxos.into_iter().filter_map(move |utxo| {
                    path.clone().map(|derivation| WalletUtxo {
                        tx_id: utxo.txid,
                        vout: utxo.vout,
                        output: TxOut {
                            value: utxo.value,
                            script_pubkey: address.script_pubkey(),
                        },
                        derivation,
                        height: utxo.status.block_height.unwrap_or(0),
                    })
                })
            })
            .collect();

        Ok(result)
    }

    /// Broadcast a signed transaction and return its txid.
    pub async fn broadcast_tx(&self, tx: &bitcoin::Transaction) -> Result<String, Error> {
        let txid = tx.compute_txid().to_string();
        self.client.broadcast(tx).await?;
        Ok(txid)
    }

    /// Estimate fee rate (sat/vB) for the given confirmation target.
    pub async fn estimate_fee_sat_vb(&self, target_blocks: u16) -> Result<f64, Error> {
        let estimates = self.get_fee_estimates().await?;
        (0..=target_blocks)
            .rev()
            .find_map(|t| estimates.get(&t).copied())
            .ok_or(Error::InvalidResponse)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn it_fee_estimate() {
        let client = EsploraAdapter::new(EsploraProvider::MempoolSpace.main_net());
        let fees = client.get_fee_estimates().await;
        match fees {
            Ok(r) => {
                println!("fees: {:?}", r);
            }
            Err(e) => {
                panic!("req failed {}", e)
            }
        }
    }

    #[tokio::test]
    async fn get_utxos_by_addresses() {
        let client = EsploraAdapter::new(EsploraProvider::MempoolSpace.main_net());
        let satochi = Address::from_str("bc1qgx3xl9f6scnh34tph2my3tytmy0m9zqurqstpp")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();

        let addreses = vec![satochi.clone()];
        let utxos = client.get_utxos_by_addresses(&addreses).await.unwrap();

        let utxos = utxos.get(&satochi).unwrap();
        println!("{:?}", utxos)
    }
}
