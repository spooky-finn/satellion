use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::{
    btc::neutrino::{EventEmitter, EventEmitterTrait},
    wallet::Wallet,
};

pub struct Session {
    pub wallet: Wallet,
    pub activated_at: DateTime<Utc>,
    pub inactivity_timeout: Duration,
}

impl Session {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            wallet,
            activated_at: Utc::now(),
            inactivity_timeout: Duration::from_mins(30),
        }
    }

    pub fn with_inactivity_timeout(mut self, inactivity_timeout: Duration) -> Self {
        self.inactivity_timeout = inactivity_timeout;
        self
    }

    /// Skip auto-lock if Bitcoin initial sync is not completed.
    /// During the sync, new UTXOs may be discovered that need to be persisted and encrypted.
    /// Leaving them unencrypted could compromise privacy and confidentiality
    /// if the computer is accessed by someone else (e.g., a third party or government).
    pub fn autolock_enabled(&self) -> bool {
        self.wallet.btc.initial_sync_done
    }

    pub fn is_expired(&self) -> bool {
        if !self.autolock_enabled() {
            return false;
        }

        self.activated_at + self.inactivity_timeout < Utc::now()
    }
}
pub type SK = Arc<Mutex<SessionKeeper>>;

pub struct SessionKeeper {
    session: Option<Session>,
    event_emitter: Option<EventEmitter>,
}

impl SessionKeeper {
    pub fn new(event_emitter: Option<EventEmitter>, monitor_interval: Option<Duration>) -> SK {
        let sk: SK = Arc::new(Mutex::new(Self {
            event_emitter: event_emitter.clone(),
            session: None,
        }));

        Self::spawn_monitor(
            sk.clone(),
            event_emitter,
            monitor_interval.unwrap_or(Duration::from_mins(1)),
        );
        sk
    }

    fn session(&mut self) -> Result<&mut Session, String> {
        match &mut self.session {
            Some(session) => {
                session.activated_at = Utc::now();
                Ok(session)
            }
            None => Err(Self::fire_expired_event(&self.event_emitter)),
        }
    }

    pub fn wallet(&mut self) -> Result<&mut Wallet, String> {
        self.session().map(|s| &mut s.wallet)
    }

    pub fn set(&mut self, session: Session) {
        self.session = Some(session);
    }

    pub fn soft_terminate(&mut self) -> bool {
        if let Some(s) = &self.session
            && s.autolock_enabled()
        {
            self.terminate();
            return true;
        }

        false
    }

    pub fn terminate(&mut self) {
        self.session = None;
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    fn spawn_monitor(sk: SK, event_emitter: Option<EventEmitter>, monitor_interval: Duration) {
        let em = event_emitter.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor_interval);
            loop {
                interval.tick().await;
                {
                    let mut sk = sk.lock().await;
                    if let Some(session) = &sk.session
                        && session.is_expired()
                    {
                        if sk.soft_terminate() {
                            Self::fire_expired_event(&em);
                            tracing::warn!("Session expired and dropped from mem");
                        };
                    }
                }
            }
        });
    }

    fn fire_expired_event(event_emitter: &Option<EventEmitter>) -> String {
        if let Some(em) = &event_emitter {
            em.session_expired();
        }
        "Session expired".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shush_rs::SecretBox;
    use std::sync::Mutex as StdMutex;
    use tokio::time::{Duration, sleep};

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    static SECRETBOX_TEST_LOCK: StdMutex<()> = StdMutex::new(());

    fn new_wallet() -> Wallet {
        let name = "test_wallet";
        let wallet = Wallet::new(
            name.to_string(),
            MNEMONIC.to_string(),
            SecretBox::new(Box::new("333".to_string())),
            None,
        )
        .expect("Failed to create test wallet");
        wallet
    }

    const MONITOR_INTERVAL: Duration = Duration::from_millis(10);

    #[tokio::test]
    async fn test_store_get() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();
        let sk = SessionKeeper::new(None, Some(MONITOR_INTERVAL));

        let mut wallet = new_wallet();
        wallet.btc.initial_sync_done = true;

        let session = Session::new(wallet).with_inactivity_timeout(Duration::from_millis(100));
        {
            let mut keeper = sk.lock().await;
            keeper.set(session);
        }

        {
            let mut keeper = sk.lock().await;
            let borrowed = keeper.session().expect("Should be able to borrow");
            assert!(!borrowed.is_expired());
        }

        // Wait for proactive expiration
        sleep(Duration::from_millis(110)).await;

        let mut keeper = sk.lock().await;
        assert!(keeper.session().is_err(), "Session should be terminated");
    }

    #[tokio::test]
    async fn test_skip_auto_lock_during_initial_sync() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();
        let sk = SessionKeeper::new(None, Some(MONITOR_INTERVAL));

        let wallet = new_wallet();
        let session = Session::new(wallet).with_inactivity_timeout(Duration::from_millis(100));
        {
            let mut keeper = sk.lock().await;
            keeper.set(session);
        }
        // Wait longer than the expiry duration
        sleep(Duration::from_millis(110)).await;
        {
            let mut keeper = sk.lock().await;
            let session_ref = keeper
                .session()
                .expect("Session should exist because sync is not done");
            assert!(
                !session_ref.is_expired(),
                "Session should not expire during initial sync"
            );
            // Mark initial sync as done
            session_ref.wallet.btc.initial_sync_done = true;
        }

        // Now that sync is done, wait for the next check
        sleep(Duration::from_millis(110)).await;
        let mut keeper = sk.lock().await;
        // Manually trigger check
        keeper.soft_terminate();
        assert!(
            keeper.session().is_err(),
            "Session should expire after initial sync is completed"
        );
    }
}
