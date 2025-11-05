use chrono::{DateTime, TimeDelta, Utc};

#[derive(Clone)]
pub struct Session {
    pub wallet_id: i32,
    pub passphrase: String,
    pub created_at: DateTime<Utc>,
    pub session_exp_duration: TimeDelta,
}

impl Session {
    pub fn new(wallet_id: i32, passphrase: String, session_exp_duration: TimeDelta) -> Self {
        Self {
            wallet_id,
            passphrase,
            created_at: Utc::now(),
            session_exp_duration,
        }
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
    use super::*;
    use std::thread;

    #[test]
    fn test_store_get() {
        let session_exp_duration = TimeDelta::seconds(2);
        let mut store = Store::new();
        let session = Session::new(1, "test".to_string(), session_exp_duration);
        store.start(session);
        assert!(store.get(1).is_some());
        assert!(store.get(2).is_none());
        assert!(store.get(1).unwrap().is_expired() == false);
        assert!(store.get(1).unwrap().wallet_id == 1);
        assert!(store.get(1).unwrap().passphrase == "test");

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(1).unwrap().is_expired() == false);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(store.get(1).is_none());
    }
}
