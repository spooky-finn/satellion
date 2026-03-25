use std::{collections::HashMap, str::FromStr};

use bitcoin::{TxOut, Txid};
use rustywallet_electrum::ElectrumError;

use crate::btc::{
    key_derivation::{DerivedAddress, KeyDerivationPath},
    utxo::Utxo,
};

pub struct ElectrumAdapter {
    client: rustywallet_electrum::ElectrumClient,
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
    pub async fn new() -> Result<Self, String> {
        let client = rustywallet_electrum::ElectrumClient::new(DNS_SEEDS[0])
            .await
            .map_err(|e| format!("fail to connect to electum server {}", e))?;
        Ok(Self { client })
    }

    pub async fn get_balances(
        &self,
        addresses: &[&str],
    ) -> Result<Vec<rustywallet_electrum::Balance>, ElectrumError> {
        Ok(self.client.get_balances(addresses).await?)
    }

    pub async fn get_utxos(
        &self,
        addresses: &[DerivedAddress],
    ) -> Result<Vec<Utxo>, ElectrumError> {
        let mut address_path_map: HashMap<String, KeyDerivationPath> = HashMap::new();
        let address_strings: Vec<String> = addresses
            .iter()
            .map(|each| {
                let addr = each.address.to_string();
                address_path_map.insert(addr.clone(), each.derive_path.clone());
                addr
            })
            .collect();

        let mut batch = rustywallet_electrum::BatchRequest::new(&self.client);
        batch = batch.utxos(address_strings.iter().map(|s| s.as_str()));
        let result = batch.execute().await?;
        let address_utxo_map = result.all_utxos();

        let result: Vec<Utxo> = address_utxo_map
            .iter()
            .filter_map(|(address, utxo)| {
                address_path_map.get(*address).map(|derivation| Utxo {
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
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::btc::{account::Account, key_derivation::Change};

    use super::*;
    use bitcoin::Address;
    use std::str::FromStr;

    #[tokio::test]
    async fn get_balances() {
        let address = Address::from_str("bc1qur8v4avm4gnqzjh7e8ny429q95y2g63ax28962")
            .unwrap()
            .assume_checked();
        let client = ElectrumAdapter::new().await.unwrap();

        let balances = client.get_balances(&[&address.to_string()]).await.unwrap();
        println!("balance {:?}", balances[0]);
    }

    #[tokio::test]
    async fn get_utxos() {
        let client = ElectrumAdapter::new().await.unwrap();
        let addresses: Vec<DerivedAddress> = vec![
            DerivedAddress {
                derive_path: Account::new_deriviation_path(0, Change::External, 0),
                address: Address::from_str("bc1qur8v4avm4gnqzjh7e8ny429q95y2g63ax28962")
                    .unwrap()
                    .assume_checked(),
            },
            DerivedAddress {
                derive_path: Account::new_deriviation_path(0, Change::External, 1),
                address: Address::from_str("bc1qgx3xl9f6scnh34tph2my3tytmy0m9zqurqstpp")
                    .unwrap()
                    .assume_checked(),
            },
        ];

        let all_utxos = client.get_utxos(&addresses).await.unwrap();
        for utxo in all_utxos.iter() {
            println!("  utxo {:?}", utxo);
        }
    }
}
