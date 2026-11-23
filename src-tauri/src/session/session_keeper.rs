use std::{
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{
    eth::walletconnect::WalletConnectState,
    event_emitter::{EventEmitter, EventEmitterTrait},
    wallet::Wallet,
};

pub struct Session {
    pub wallet: Wallet,
    pub activated_at: DateTime<Utc>,
    pub inactivity_timeout: Duration,
    pub(crate) walletconnect: WalletConnectState,
    access: SessionAccess,
}

/// Shared handle for invalidating wallet session access without acquiring the
/// async [`SessionKeeper`] lock. Clones share the same revocation state.
///
/// [`Self::revoke`] advances the session generation and cancels the previous
/// generation's token, waking async operations waiting for revocation. The
/// generation check rejects unlock attempts and signing requests from before
/// revocation. The revocation mutex stays held from that check through local
/// signing.
#[derive(Clone, Default)]
pub struct SessionRevocation(Arc<StdMutex<SessionAccess>>);

#[derive(Clone, Default)]
pub(crate) struct SessionAccess {
    generation: u64,
    cancelled: CancellationToken,
}

impl SessionAccess {
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }

    pub(crate) async fn cancelled(&self) {
        self.cancelled.cancelled().await;
    }

    fn ensure_generation(&self, generation: u64, error: &str) -> Result<(), String> {
        if self.generation != generation {
            return Err(error.into());
        }
        Ok(())
    }
}

impl SessionRevocation {
    fn advance(access: &mut SessionAccess) {
        access.cancelled.cancel();
        access.generation += 1;
        access.cancelled = CancellationToken::new();
    }

    pub fn revoke(&self) {
        Self::advance(&mut self.0.lock().expect("session access lock"));
    }

    pub(crate) fn generation(&self) -> u64 {
        self.0.lock().expect("session access lock").generation
    }

    fn start(&self, generation: u64) -> Result<SessionAccess, String> {
        let mut access = self.0.lock().expect("session access lock");
        access.ensure_generation(generation, "The wallet locked during unlock")?;
        Self::advance(&mut access);
        Ok(access.clone())
    }

    fn while_current<T>(
        &self,
        expected: &SessionAccess,
        action: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let access = self.0.lock().expect("session access lock");
        access.ensure_generation(
            expected.generation,
            "The wallet locked or changed before signing",
        )?;
        action()
    }
}

impl Session {
    pub fn new(wallet: Wallet, inactivity_timeout: Duration) -> Self {
        Self {
            wallet,
            activated_at: Utc::now(),
            inactivity_timeout,
            walletconnect: WalletConnectState::default(),
            access: SessionAccess::default(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.access.is_cancelled() || self.activated_at + self.inactivity_timeout <= Utc::now()
    }

    pub(crate) fn access(&self) -> &SessionAccess {
        &self.access
    }
}
pub type SK = Arc<Mutex<SessionKeeper>>;

pub struct SessionKeeper {
    session: Option<Session>,
    event_emitter: Option<EventEmitter>,
    revocation: SessionRevocation,
}

impl SessionKeeper {
    pub fn new(event_emitter: Option<EventEmitter>, monitor_interval: Option<Duration>) -> SK {
        Self::with_revocation(
            event_emitter,
            monitor_interval,
            SessionRevocation::default(),
        )
    }

    pub fn with_revocation(
        event_emitter: Option<EventEmitter>,
        monitor_interval: Option<Duration>,
        revocation: SessionRevocation,
    ) -> SK {
        let sk: SK = Arc::new(Mutex::new(Self {
            event_emitter: event_emitter.clone(),
            session: None,
            revocation,
        }));

        Self::spawn_monitor(
            sk.clone(),
            event_emitter,
            monitor_interval.unwrap_or(Duration::from_mins(1)),
        );
        sk
    }

    fn session(&mut self) -> Result<&mut Session, String> {
        let session = self.unexpired_session()?;
        session.activated_at = Utc::now();
        Ok(session)
    }

    pub fn wallet(&mut self) -> Result<&mut Wallet, String> {
        self.session().map(|s| &mut s.wallet)
    }

    pub(crate) fn unexpired_session(&mut self) -> Result<&mut Session, String> {
        self.session
            .as_mut()
            .filter(|session| !session.is_expired())
            .ok_or_else(|| Self::fire_expired_event(&self.event_emitter))
    }

    /// Run a synchronous wallet action while its original session is still current.
    /// The caller holds the keeper lock and this method holds the revocation lock
    /// through the action, so a concurrent wallet lock cannot race local signing.
    pub(crate) fn with_current_session<T>(
        &mut self,
        expected: &SessionAccess,
        action: impl FnOnce(&mut Session) -> Result<T, String>,
    ) -> Result<T, String> {
        let revocation = self.revocation.clone();
        let session = self.unexpired_session()?;
        revocation.while_current(expected, || {
            if session.is_expired() {
                return Err("The wallet locked before signing".into());
            }
            action(session)
        })
    }

    pub(crate) fn clear_walletconnect(&mut self) {
        if let Some(session) = &mut self.session {
            session.walletconnect = WalletConnectState::default();
        }
    }

    pub fn set(&mut self, mut session: Session) {
        let mut access = self.revocation.0.lock().expect("session access lock");
        SessionRevocation::advance(&mut access);
        session.access = access.clone();
        self.session = Some(session);
    }

    pub(crate) fn set_at(&mut self, mut session: Session, generation: u64) -> Result<(), String> {
        session.access = self.revocation.start(generation)?;
        self.session = Some(session);
        Ok(())
    }

    pub fn drop_revoked(&mut self) {
        if self
            .session
            .as_ref()
            .is_some_and(|session| session.access.is_cancelled())
        {
            self.session = None;
        }
    }

    pub fn soft_terminate(&mut self) -> bool {
        if self.session.is_some() {
            self.terminate();
            return true;
        }

        false
    }

    pub fn terminate(&mut self) {
        if self
            .session
            .as_ref()
            .is_some_and(|session| !session.access.is_cancelled())
        {
            self.revocation.revoke();
        }
        self.session = None;
    }

    pub fn has_session(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| !session.is_expired())
    }

    fn spawn_monitor(sk: SK, event_emitter: Option<EventEmitter>, monitor_interval: Duration) {
        let em = event_emitter.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(monitor_interval);
            loop {
                interval.tick().await;
                {
                    let mut sk = sk.lock().await;
                    if let Some(session) = &sk.session
                        && session.is_expired()
                        && sk.soft_terminate()
                    {
                        Self::fire_expired_event(&em);
                        tracing::warn!("Session expired and dropped from mem");
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
    use std::sync::Mutex as StdMutex;

    use tokio::time::{Duration, sleep};

    use super::*;
    use crate::config::Config;

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    static SECRETBOX_TEST_LOCK: StdMutex<()> = StdMutex::new(());

    fn new_wallet() -> Wallet {
        let name = "test_wallet";
        let config = Config::new();
        let wallet = Wallet::new(
            config,
            name.to_string(),
            MNEMONIC.to_string(),
            "333".to_string(),
            None,
        )
        .expect("Failed to create test wallet");
        wallet
    }

    const MONITOR_INTERVAL: Duration = Duration::from_millis(10);

    #[tokio::test]
    async fn signing_requires_the_original_session_after_replacement() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();
        let sk = SessionKeeper::new(None, None);
        let mut keeper = sk.lock().await;
        let timeout = Duration::from_secs(120);
        keeper.set(Session::new(new_wallet(), timeout));
        let previous = keeper.unexpired_session().unwrap().access().clone();

        keeper.set(Session::new(new_wallet(), timeout));
        let current = keeper.unexpired_session().unwrap().access().clone();
        assert!(previous.is_cancelled());
        assert_eq!(
            keeper
                .with_current_session::<()>(&previous, |_| panic!("stale session signed"))
                .unwrap_err(),
            "The wallet locked or changed before signing"
        );
        keeper.with_current_session(&current, |_| Ok(())).unwrap();
    }

    #[tokio::test]
    async fn test_store_get() {
        let _guard = SECRETBOX_TEST_LOCK.lock().unwrap();
        let sk = SessionKeeper::new(None, Some(MONITOR_INTERVAL));

        let wallet = new_wallet();

        let session = Session::new(wallet, Duration::from_millis(100));
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
}
