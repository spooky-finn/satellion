pub use std::sync::Arc;

use tauri::Listener;
use tokio::sync::Mutex;

use crate::{
    event_emitter::{EventEmitter, EventEmitterTrait},
    session::{SessionKeeper, SessionRevocation},
    system::session_monitor,
};

pub fn init_session_triggers(
    app_handle: &tauri::AppHandle,
    sk: Arc<Mutex<SessionKeeper>>,
    revocation: SessionRevocation,
    event_emitter: Arc<EventEmitter>,
) {
    // Listener for session lock
    {
        let sk = sk.clone();
        let em = event_emitter.clone();
        app_handle.listen(session_monitor::SYS_SESSION_LOCKED_EVENT, move |_| {
            revocation.revoke();
            em.session_expired();
            let sk = sk.clone();
            tauri::async_runtime::spawn(async move {
                sk.lock().await.drop_revoked();
            });
        });
    }

    // Listener for session unlock
    {
        let sk = sk.clone();
        let em = event_emitter.clone();
        app_handle.listen(session_monitor::SYS_SESSION_UNLOCKED_EVENT, move |_| {
            let sk = sk.clone();
            let emmiter = em.clone();
            tauri::async_runtime::spawn(async move {
                let sk = sk.lock().await;
                // If no session exist just emit event to redirect UI
                if !sk.has_session() {
                    emmiter.session_expired();
                }
            });
        });
    }
}
