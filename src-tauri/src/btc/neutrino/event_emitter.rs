use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_specta::{Events, collect_events};
use tracing::error;

pub const EVENT_HEIGHT_UPDATE: &str = "btc_sync";
pub const EVENT_SYNC_PROGRESS: &str = "btc_sync_progress";
pub const EVENT_SYNC_WARNING: &str = "btc_sync_warning";
pub const EVENT_SYNC_NEW_UTXO: &str = "btc_sync_new_utxo";

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

impl EventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn height_updated(&self, height: u32, status: HeightUpdateStatus) {
        self.app
            .emit(
                EVENT_HEIGHT_UPDATE,
                SyncHeightUpdateEvent { height, status },
            )
            .unwrap_or_else(|e| error!("fail to emit event {}", e));
    }

    pub fn cf_sync_progress(&self, pct: f32) {
        self.app
            .emit(EVENT_SYNC_PROGRESS, SyncProgressEvent { progress: pct })
            .unwrap_or_else(|e| error!("fail to emit event {}", e))
    }

    pub fn node_warning(&self, msg: String) {
        self.app
            .emit(EVENT_SYNC_WARNING, SyncWarningEvent { msg })
            .unwrap();
    }

    pub fn new_utxo(&self, value: String) {
        self.app
            .emit(EVENT_SYNC_NEW_UTXO, SyncNewUtxoEvent { value })
            .unwrap();
    }
}
