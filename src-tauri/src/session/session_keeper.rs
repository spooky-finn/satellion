use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};

use crate::{btc::neutrino::EventEmitter, wallet::Wallet};

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

    /// Skip auto-lock if Bitcoin initial sync is not completed.
    /// During the sync, new UTXOs may be discovered that need to be persisted and encrypted.
    /// Leaving them unencrypted could compromise privacy and confidentiality
    /// if the computer is accessed by someone else (e.g., a third party or government).
    pub fn autolock_disabled(&self) -> bool {
        self.wallet.btc.initial_sync_done == false
    }

    pub fn is_expired(&self) -> bool {
        if self.autolock_disabled() {
            return false;
        }

        self.created_at + self.session_exp_duration < Utc::now()
    }
}
/// Session Keeper Type
pub type SK = Arc<tokio::sync::Mutex<SessionKeeper>>;

/// Session Keeper
#[derive(Default)]
pub struct SessionKeeper {
    session: Option<Session>,
    event_emitter: Option<EventEmitter>,
}

impl SessionKeeper {
    pub fn new(event_emitter: Option<EventEmitter>) -> Self {
        Self {
            session: None,
            event_emitter,
        }
    }

    pub fn borrow(&mut self) -> Result<&mut Session, String> {
        if let Some(session) = &self.session {
            if session.is_expired() {
                self.session = None;

                if let Some(event_emitter) = &self.event_emitter {
                    let _ = event_emitter.session_expired();
                }
                return Err("Session has expired".to_string());
            }

            Ok(self
                .session
                .as_mut()
                .expect("fail to borrow session: probably expired"))
        } else {
            if let Some(event_emitter) = &self.event_emitter {
                let _ = event_emitter.session_expired();
            }
            Err("Session has expired".to_string())
        }
    }

    pub fn set(&mut self, session: Session) {
        self.session = Some(session);
    }

    pub fn soft_terminate(&mut self) {
        if let Some(s) = &self.session {
            if !s.autolock_disabled() {
                self.terminate();
            }
        }
    }

    pub fn terminate(&mut self) {
        self.session = None;
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }
}

#[cfg(test)]
mod tests {
    use shush_rs::SecretBox;

    use super::*;
    use std::{sync::Mutex, thread, time::Duration};

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    static SECRETBOX_TEST_LOCK: Mutex<()> = Mutex::new(());
    fn make_secret_box() -> SecretBox<String> {
        SecretBox::new(Box::new("333".to_string()))
    }

    #[test]
    fn test_store_get() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();

        let session_exp_duration = TimeDelta::seconds(2);
        let mut sk = SessionKeeper::new(None);
        let name = "test_wallet";
        let mut wallet = Wallet::new(
            name.to_string(),
            MNEMONIC.to_string(),
            make_secret_box(),
            None,
        )
        .expect("Failed to create test wallet");
        wallet.btc.initial_sync_done = true;

        let session = Session::new(wallet, session_exp_duration);
        sk.set(session);

        assert!(sk.borrow().is_ok());
        assert!(sk.borrow().unwrap().is_expired() == false);
        assert!(sk.borrow().unwrap().wallet.name == name);

        thread::sleep(Duration::from_secs(1));
        assert!(sk.borrow().unwrap().is_expired() == false);

        thread::sleep(Duration::from_secs(1));
        assert!(sk.borrow().is_err());
    }

    #[test]
    fn test_skip_auto_lock_during_initial_sync() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();

        let session_exp_duration = TimeDelta::seconds(1);
        let mut sk = SessionKeeper::new(None);
        let wallet = Wallet::new(
            "sync_wallet".to_string(),
            MNEMONIC.to_string(),
            make_secret_box(),
            None,
        )
        .expect("Failed to create test wallet");
        let session = Session::new(wallet, session_exp_duration);
        sk.set(session);

        // Even after the duration has passed, session should NOT expire
        thread::sleep(Duration::from_secs(2));
        let session_ref = sk.borrow().expect("Session should exist");
        assert!(
            !session_ref.is_expired(),
            "Session should not expire during initial sync"
        );
        // Mark initial sync as done
        session_ref.wallet.btc.initial_sync_done = true;
        // After this, expiration should work normally
        thread::sleep(Duration::from_secs(1));
        assert!(
            sk.borrow().is_err(),
            "Session should expire after initial sync is completed"
        );
    }
}
