use crate::{
    config::Chain,
    session::chain_data::{BitcoinSession, EthereumSession},
};
use chain_data::ChainData;
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::HashMap;
pub mod chain_data;

#[derive(Clone)]
pub struct Session {
    pub wallet_id: i32,
    pub created_at: DateTime<Utc>,
    pub session_exp_duration: TimeDelta,
    chain_data: HashMap<Chain, ChainData>,
}

impl Session {
    pub fn new(wallet_id: i32, session_exp_duration: TimeDelta) -> Self {
        Self {
            wallet_id,
            created_at: Utc::now(),
            session_exp_duration,
            chain_data: HashMap::new(),
        }
    }

    pub fn add_chain_data(&mut self, chain: Chain, data: ChainData) {
        self.chain_data.insert(chain, data);
    }

    pub fn get_bitcoin_session(&self) -> Option<&BitcoinSession> {
        self.chain_data
            .get(&Chain::Bitcoin)
            .and_then(|data| data.as_bitcoin())
    }

    pub fn get_ethereum_session(&self) -> Option<&EthereumSession> {
        self.chain_data
            .get(&Chain::Ethereum)
            .and_then(|data| data.as_ethereum())
    }

    pub fn is_expired(&self) -> bool {
        self.created_at + self.session_exp_duration < Utc::now()
    }
}

#[derive(Default)]
pub struct Store {
    session: Option<Session>,
}

impl Store {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn get(&mut self, wallet_id: i32) -> Option<Session> {
        match &self.session {
            None => None,
            Some(session) if session.is_expired() => {
                self.session = None;
                None
            }
            Some(session) if session.wallet_id != wallet_id => None,
            Some(session) => Some(session.clone()),
        }
    }

    pub fn start(&mut self, session: Session) {
        self.session = Some(session);
    }

    pub fn end(&mut self) {
        self.session = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::config::CONFIG;
    use alloy_signer_local::PrivateKeySigner;
    use bitcoin::bip32::Xpriv;

    use super::*;
    use std::thread;

    #[test]
    fn test_store_get() {
        let session_exp_duration = TimeDelta::seconds(2);
        let mut store = Store::new();
        let session = Session::new(1, session_exp_duration);
        store.start(session);
        assert!(store.get(1).is_some());
        assert!(store.get(2).is_none());
        assert!(store.get(1).unwrap().is_expired() == false);
        assert!(store.get(1).unwrap().wallet_id == 1);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(1).unwrap().is_expired() == false);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(1).is_none());
    }

    #[test]
    fn test_session_add_chain_data() {
        use crate::session::chain_data::{BitcoinSession, EthereumSession};
        use Chain;

        let mut session = Session::new(1, TimeDelta::hours(1));

        assert!(session.get_bitcoin_session().is_none());
        assert!(session.get_ethereum_session().is_none());

        let btc_session = BitcoinSession {
            xprv: Xpriv::new_master(
                CONFIG.bitcoin.network(),
                &bitcoin::key::rand::random::<[u8; 32]>().to_vec(),
            )
            .unwrap(),
        };
        let eth_session = EthereumSession {
            signer: PrivateKeySigner::random(),
        };

        session.add_chain_data(Chain::Bitcoin, ChainData::from(btc_session));
        session.add_chain_data(Chain::Ethereum, ChainData::from(eth_session));

        assert!(session.get_bitcoin_session().is_some());
        assert!(session.get_ethereum_session().is_some());
    }
}
