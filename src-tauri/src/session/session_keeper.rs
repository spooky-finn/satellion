use chrono::{DateTime, TimeDelta, Utc};
use shush_rs::SecretBox;

use crate::wallet::Wallet;

pub struct Session {
    pub wallet: Wallet,
    pub passphrase: SecretBox<String>,
    pub created_at: DateTime<Utc>,
    pub session_exp_duration: TimeDelta,
}

impl Session {
    pub fn new(wallet: Wallet, passphrase: String, session_exp_duration: TimeDelta) -> Self {
        Self {
            wallet,
            passphrase: SecretBox::new(Box::new(passphrase)),
            created_at: Utc::now(),
            session_exp_duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at + self.session_exp_duration < Utc::now()
    }
}

pub type AppSession = tokio::sync::Mutex<SessionKeeper>;

#[derive(Default)]
pub struct SessionKeeper {
    session: Option<Session>,
}

impl SessionKeeper {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn get(&mut self, wallet_name: &str) -> Result<&mut Session, String> {
        if let Some(session) = &self.session {
            if session.is_expired() || session.wallet.name != wallet_name {
                self.session = None;
                return Err("Session has expired".to_string());
            }

            Ok(self
                .session
                .as_mut()
                .expect("fail to borrow session: probably expired"))
        } else {
            Err("Session has expired".to_string())
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
        let mut session_keeper = SessionKeeper::new();
        let name = "test_wallet";
        let wallet = Wallet::new(
            name.to_string(),
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
        ).expect("Failed to create test wallet");

        let session = Session::new(wallet, "1111".to_string(), session_exp_duration);
        session_keeper.start(session);

        assert!(session_keeper.get(name).is_ok());
        assert!(session_keeper.get(name).unwrap().is_expired() == false);
        assert!(session_keeper.get(name).unwrap().wallet.name == name);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(session_keeper.get(name).unwrap().is_expired() == false);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(session_keeper.get(name).is_err());
    }
}
