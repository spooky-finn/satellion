use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_specta::{Events, collect_events};

use mockall::{automock, predicate::*};

#[automock]
pub trait EventEmitterTrait: Send + Sync {
    fn session_expired(&self);
}

pub const EVENT_SESSION_EXPIRED: &str = "session_expired";

pub fn list_events() -> Events {
    collect_events![]
}

/** Event emitter for UI */
#[derive(Clone)]
pub struct EventEmitter {
    app: AppHandle,
}

impl EventEmitterTrait for EventEmitter {
    fn session_expired(&self) {
        self.emit(EVENT_SESSION_EXPIRED, ());
    }
}

impl EventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) {
        if let Err(e) = self.app.emit(event, payload) {
            tracing::error!(
                event = event,
                error = %e,
                "failed to emit tauri event"
            );
        }
    }
}
