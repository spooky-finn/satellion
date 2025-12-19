use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};

use crate::{
    config::Chain,
    session::{BitcoinSession, ChainSession, EthereumSession},
};

#[derive(Clone)]
pub struct Session {
    pub wallet_name: String,
    pub created_at: DateTime<Utc>,
    pub session_exp_duration: TimeDelta,
    chain_data: HashMap<Chain, ChainSession>,
}

impl Session {
    pub fn new(wallet_name: String, session_exp_duration: TimeDelta) -> Self {
        Self {
            wallet_name,
            created_at: Utc::now(),
            session_exp_duration,
            chain_data: HashMap::new(),
        }
    }

    pub fn add_chain_data(&mut self, chain: Chain, data: ChainSession) {
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

    pub fn get(&mut self, wallet_name: &str) -> Option<Session> {
        match &self.session {
            None => None,
            Some(session) if session.is_expired() => {
                self.session = None;
                None
            }
            Some(session) if session.wallet_name != wallet_name => None,
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
        let wallet_name = "test_wallet".to_string();
        let session = Session::new(wallet_name.clone(), session_exp_duration);
        store.start(session);
        assert!(store.get(&wallet_name).is_some());
        assert!(store.get("other_wallet").is_none());
        assert!(store.get(&wallet_name).unwrap().is_expired() == false);
        assert!(store.get(&wallet_name).unwrap().wallet_name == wallet_name);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(&wallet_name).unwrap().is_expired() == false);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(&wallet_name).is_none());
    }

    #[test]
    fn test_session_add_chain_data() {
        use crate::session::chain_state::{BitcoinSession, EthereumSession};
        use Chain;

        let mut session = Session::new("test_wallet".to_string(), TimeDelta::hours(1));

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

        session.add_chain_data(Chain::Bitcoin, ChainSession::from(btc_session));
        session.add_chain_data(Chain::Ethereum, ChainSession::from(eth_session));

        assert!(session.get_bitcoin_session().is_some());
        assert!(session.get_ethereum_session().is_some());
    }
}
