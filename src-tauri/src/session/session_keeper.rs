use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};

use crate::wallet::Wallet;

pub struct Session {
    pub wallet: Wallet,
    pub created_at: DateTime<Utc>,
    pub session_exp_duration: TimeDelta,
}

impl Session {
    pub fn new(wallet: Wallet, session_exp_duration: TimeDelta) -> Self {
        Self {
            wallet,
            created_at: Utc::now(),
            session_exp_duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at + self.session_exp_duration < Utc::now()
    }
}
/// Session Keeper Type
pub type SK = Arc<tokio::sync::Mutex<SessionKeeper>>;

/// Session Keeper
#[derive(Default)]
pub struct SessionKeeper {
    session: Option<Session>,
}

impl SessionKeeper {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn take_session(&mut self, wallet_name: &str) -> Result<&mut Session, String> {
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
    use shush_rs::SecretBox;

    use super::*;
    use std::thread;

    #[test]
    fn test_store_get() {
        let session_exp_duration = TimeDelta::seconds(2);
        let mut sk = SessionKeeper::new();
        let name = "test_wallet";
        let wallet = Wallet::new(
            name.to_string(),
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            SecretBox::new(Box::new("333".to_string()))
        ).expect("Failed to create test wallet");

        let session = Session::new(wallet, session_exp_duration);
        sk.start(session);

        assert!(sk.take_session(name).is_ok());
        assert!(sk.take_session(name).unwrap().is_expired() == false);
        assert!(sk.take_session(name).unwrap().wallet.name == name);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(sk.take_session(name).unwrap().is_expired() == false);

        thread::sleep(std::time::Duration::from_secs(1));
        assert!(sk.take_session(name).is_err());
    }
}
