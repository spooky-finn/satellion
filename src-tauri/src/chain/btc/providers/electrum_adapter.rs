use std::{str::FromStr, time::Duration};

use bitcoin::{TxOut, Txid};
use rustywallet_electrum::{Balance, ClientConfig, ElectrumClient, ElectrumError};
use tokio::sync::OnceCell;

use crate::chain::btc::{account::AddressPathMap, config::BitcoinConfig, utxo::Utxo};

#[derive(Default)]
pub struct ElectrumAdapter {
    connection: OnceCell<ElectrumClient>,
    config: BitcoinConfig,
}

/// DNS seeds for discovering Electrum servers.
const DNS_SEEDS: &[&str] = &[
    // Bitcoin mainnet Electrum DNS seeds
    "electrum.blockstream.info",
    "electrum1.bluewallet.io",
    "electrum2.bluewallet.io",
    "bitcoin.aranguren.org",
    "electrum.bitaroo.net",
    "electrum.emzy.de",
    "electrum.hodlister.co",
];

impl ElectrumAdapter {
    pub fn new(config: BitcoinConfig) -> Self {
        ElectrumAdapter {
            connection: OnceCell::default(),
            config,
        }
    }

    async fn create_client(&self) -> Result<ElectrumClient, ElectrumError> {
        match self.config.regtest {
            true => {
                ElectrumClient::with_config(ClientConfig {
                    server: "127.0.0.1".to_string(),
                    port: 50002,
                    use_tls: false,
                    timeout: Duration::from_secs(5),
                    retry_count: 2,
                    retry_delay: Duration::from_secs(1),
                    skip_tls_verify: true,
                })
                .await
            }
            false => ElectrumClient::new(DNS_SEEDS[0]).await,
        }
    }

    /// Ensure the connection is initialized before use
    async fn get_conn(&self) -> Result<&ElectrumClient, ElectrumError> {
        self.connection
            .get_or_try_init(|| async {
                self.create_client().await.map_err(|e| {
                    tracing::error!("conn err {}", e);
                    ElectrumError::Disconnected
                })
            })
            .await
    }

    pub async fn get_balances(&self, addresses: &[&str]) -> Result<Vec<Balance>, ElectrumError> {
        let conn = self.get_conn().await?;
        conn.get_balances(addresses).await
    }

    /// Get estimated fee rate (satoshis per byte).
    ///
    /// # Arguments
    /// * `blocks` - Target confirmation blocks (e.g., 1, 6, 144)
    pub async fn estimate_fee(&self, blocks: u32) -> Result<f64, ElectrumError> {
        let conn = self.get_conn().await?;
        conn.estimate_fee(blocks).await.map(|btc_per_kvb| {
            // Just move the decimal to convert BTC/kvB -> sat/vB
            btc_per_kvb * 100_000.0
        })
    }

    pub async fn get_utxos(
        &self,
        address_path_map: AddressPathMap,
    ) -> Result<Vec<Utxo>, ElectrumError> {
        let conn = self.get_conn().await?;
        let batch = rustywallet_electrum::BatchRequest::new(conn)
            .utxos(address_path_map.keys().map(|s| s.to_string()));

        let result = batch.execute().await?;
        let address_utxo_map = result.all_utxos();

        let result: Vec<Utxo> = address_utxo_map
            .iter()
            .filter_map(|(address_str, utxo)| {
                address_str
                    .parse::<bitcoin::Address<_>>()
                    .ok()
                    .and_then(|address| address.require_network(self.config.network()).ok())
                    .and_then(|address| {
                        address_path_map.get(&address).map(|derivation| Utxo {
                            tx_id: Txid::from_str(&utxo.txid).expect("invalid txid"),
                            vout: utxo.vout as usize,
                            output: TxOut {
                                value: bitcoin::Amount::from_sat(utxo.value),
                                script_pubkey: bitcoin::ScriptBuf::new(),
                            },
                            derivation: derivation.clone(),
                            height: utxo.height as u32,
                        })
                    })
            })
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        chain::btc::key_derivation::{Change, DerivedAddress, KeyDerivationPath, Proposal},
        mnemonic::TEST_MNEMONIC,
    };

    use super::*;
    use bip39::{Language, Mnemonic};
    use bitcoin::bip32::Xpriv;

    fn derive_address_from_test_mnemonic(index: u32, change: Change) -> DerivedAddress {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let xpriv = Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).unwrap();
        let path =
            KeyDerivationPath::new(Proposal::Bip86, bitcoin::Network::Regtest, 0, change, index);
        let child = path.derive(&xpriv).unwrap();
        DerivedAddress {
            derive_path: path,
            address: child.taproot_address,
        }
    }

    fn get_config() -> BitcoinConfig {
        let mut c = BitcoinConfig::default();
        c.regtest = true;
        c
    }

    #[tokio::test]
    async fn get_balances() {
        let config = get_config();
        let client = ElectrumAdapter::new(config);
        let address = derive_address_from_test_mnemonic(0, Change::External);

        let balances = client
            .get_balances(&[&address.address.to_string()])
            .await
            .unwrap();
        println!("balance {:?}", balances[0]);
    }

    #[tokio::test]
    async fn get_utxos() {
        let config = get_config();
        let client = ElectrumAdapter::new(config);
        let addr0 = derive_address_from_test_mnemonic(0, Change::External);
        let addr1 = derive_address_from_test_mnemonic(1, Change::External);
        let addresses = HashMap::from([
            (addr0.address, addr0.derive_path),
            (addr1.address, addr1.derive_path),
        ]);

        let all_utxos = client.get_utxos(addresses).await.unwrap();
        for utxo in all_utxos.iter() {
            println!("  utxo {:?}", utxo);
        }
    }
}
