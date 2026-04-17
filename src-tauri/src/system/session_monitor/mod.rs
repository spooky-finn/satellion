pub mod triggers;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{event_emitter::EventEmitter, session::SessionKeeper};

pub const SYS_SESSION_LOCKED_EVENT: &str = "system:session-locked";
pub const SYS_SESSION_UNLOCKED_EVENT: &str = "system:session-unlocked";

#[cfg(target_os = "macos")]
mod macos;

pub fn init(
    app: &tauri::AppHandle,
    sk: Arc<Mutex<SessionKeeper>>,
    event_emitter: Arc<EventEmitter>,
) {
    #[cfg(target_os = "macos")]
    macos::init_session_events(app.clone());
    triggers::init_session_triggers(app, sk, event_emitter);
}
