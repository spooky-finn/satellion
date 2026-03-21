use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_specta::{Events, collect_events};

use mockall::{automock, predicate::*};

#[automock]
pub trait EventEmitterTrait: Send + Sync {
    fn height_updated(&self, height: u32, status: HeightUpdateStatus);
    fn cf_sync_progress(&self, pct: f32);
    fn node_warning(&self, msg: String);
    fn new_utxo(&self, value: u64, total: u64);
    fn session_expired(&self);
}

pub const EVENT_HEIGHT_UPDATE: &str = "btc_sync";
pub const EVENT_SYNC_PROGRESS: &str = "btc_sync_progress";
pub const EVENT_SYNC_WARNING: &str = "btc_sync_warning";
pub const EVENT_SYNC_NEW_UTXO: &str = "btc_sync_new_utxo";
pub const EVENT_SESSION_EXPIRED: &str = "session_expired";

#[derive(Debug, Clone, Serialize, Type)]
pub enum HeightUpdateStatus {
    #[serde(rename = "in progress")]
    Progress,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
struct SyncHeightUpdateEvent {
    status: HeightUpdateStatus,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
struct SyncProgressEvent {
    progress: f32,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
struct SyncWarningEvent {
    msg: String,
}

#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
struct SyncNewUtxoEvent {
    value: String,
    total: String,
}

pub fn list_events() -> Events {
    collect_events![
        SyncHeightUpdateEvent,
        SyncProgressEvent,
        SyncWarningEvent,
        SyncNewUtxoEvent
    ]
}

/** Event emitter for UI */
#[derive(Clone)]
pub struct EventEmitter {
    app: AppHandle,
}

impl EventEmitterTrait for EventEmitter {
    fn height_updated(&self, height: u32, status: HeightUpdateStatus) {
        self.emit(
            EVENT_HEIGHT_UPDATE,
            SyncHeightUpdateEvent { height, status },
        );
    }

    fn cf_sync_progress(&self, pct: f32) {
        self.emit(EVENT_SYNC_PROGRESS, SyncProgressEvent { progress: pct });
    }

    fn node_warning(&self, msg: String) {
        self.emit(EVENT_SYNC_WARNING, SyncWarningEvent { msg });
    }

    fn new_utxo(&self, value: u64, total: u64) {
        self.emit(
            EVENT_SYNC_NEW_UTXO,
            SyncNewUtxoEvent {
                value: value.to_string(),
                total: total.to_string(),
            },
        );
    }

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
