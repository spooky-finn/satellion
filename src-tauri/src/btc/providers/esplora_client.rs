use std::collections::HashMap;

use bitcoin::Address;
use esplora_client::{AsyncClient, Builder, Error, Utxo, r#async::DefaultSleeper};
use futures::future::join_all;

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
}

// Esplora API is a RESTful HTTP interface for querying Bitcoin blockchain data.
// It is developed by Blockstream and powers the public Blockstream Explorer.
// It allows clients (wallets, services, indexers) to fetch blockchain state without running a full node with a custom indexer.
pub struct EsploraClient {
    client: AsyncClient<DefaultSleeper>,
}

impl EsploraClient {
    pub fn new(base_url: &str) -> Self {
        let client = Builder::new(base_url)
            .build_async()
            .expect("fail to create EsploraClient");
        Self { client }
    }

    /// Get a map where the key is the confirmation target (in number of
    /// blocks) and the value is the estimated feerate (in sat/vB).
    pub async fn get_fee_estimates(&self) -> Result<HashMap<u16, f64>, Error> {
        Ok(self.client.get_fee_estimates().await?)
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
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn it_fee_estimate() {
        let client = EsploraClient::new(EsploraProvider::MempoolSpace.main_net());
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
        let client = EsploraClient::new(EsploraProvider::MempoolSpace.main_net());
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
