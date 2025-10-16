use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub chain_height: Arc<RwLock<Option<u32>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            chain_height: Arc::new(RwLock::new(None)),
        }
    }
}
