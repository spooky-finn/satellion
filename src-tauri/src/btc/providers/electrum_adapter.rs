use std::str::FromStr;

use bitcoin::{TxOut, Txid};
use rustywallet_electrum::{Balance, ElectrumClient, ElectrumError};

use crate::btc::{account::AddressPathMap, utxo::Utxo};

pub struct ElectrumAdapter {
    transport: ElectrumClient,
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
        let client = ElectrumClient::new(DNS_SEEDS[0])
            .await
            .map_err(|e| format!("fail to connect to electum server {}", e))?;
        Ok(Self { transport: client })
    }

    pub async fn get_balances(&self, addresses: &[&str]) -> Result<Vec<Balance>, ElectrumError> {
        self.transport.get_balances(addresses).await
    }

    pub async fn get_utxos(
        &self,
        address_path_map: AddressPathMap,
    ) -> Result<Vec<Utxo>, ElectrumError> {
        let batch = rustywallet_electrum::BatchRequest::new(&self.transport)
            .utxos(address_path_map.keys().map(|s| s.to_string()));

        let result = batch.execute().await?;
        let address_utxo_map = result.all_utxos();

        let result: Vec<Utxo> = address_utxo_map
            .iter()
            .filter_map(|(address_str, utxo)| {
                address_str
                    .parse::<bitcoin::Address<_>>()
                    .ok()
                    .and_then(|address| address.require_network(bitcoin::Network::Bitcoin).ok())
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
        btc::{
            account::Account,
            key_derivation::{Change, DerivedAddress},
        },
        mnemonic::TEST_MNEMONIC,
    };

    use super::*;
    use bip39::{Language, Mnemonic};
    use bitcoin::bip32::Xpriv;

    fn derive_address_from_test_mnemonic(index: u32, change: Change) -> DerivedAddress {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let xpriv = Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).unwrap();
        let path = Account::new_deriviation_path(0, change, index);
        let child = path.derive(&xpriv).unwrap();
        DerivedAddress {
            derive_path: path,
            address: child.address,
        }
    }

    #[tokio::test]
    async fn get_balances() {
        let client = ElectrumAdapter::new().await.unwrap();
        let address = derive_address_from_test_mnemonic(0, Change::External);

        let balances = client
            .get_balances(&[&address.address.to_string()])
            .await
            .unwrap();
        println!("balance {:?}", balances[0]);
    }

    #[tokio::test]
    async fn get_utxos() {
        let client = ElectrumAdapter::new().await.unwrap();
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
