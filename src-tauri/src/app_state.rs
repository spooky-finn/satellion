use std::sync::{Arc, Mutex};

pub struct AppState {
    pub chain_height: Arc<Mutex<u32>>,
    pub sync_completed: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            chain_height: Arc::new(Mutex::new(0)),
            sync_completed: Arc::new(Mutex::new(false)),
        }
    }
}
