use tauri::async_runtime::JoinHandle;
use tokio_util::sync::CancellationToken;

/// Manages the lifecycle state of a Neutrino node instance
pub struct LifecycleState {
    pub running_for_wallet: Option<String>,
    pub cancel_token: Option<CancellationToken>,
    pub task: Option<JoinHandle<()>>,
}

impl LifecycleState {
    pub fn new() -> Self {
        Self {
            running_for_wallet: None,
            cancel_token: None,
            task: None,
        }
    }

    /// Check if node is running for the specified wallet
    pub fn is_running_for(&self, wallet_name: &str) -> bool {
        self.running_for_wallet.as_deref() == Some(wallet_name)
    }

    /// Stop the currently running node instance
    pub fn stop_current(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            tracing::info!("Stopping neutrino for previous wallet");
            token.cancel();
        }

        if let Some(task) = self.task.take() {
            task.abort(); // defensive cleanup
        }
    }

    /// Start a new node instance for a wallet
    pub fn start_for_wallet(
        &mut self,
        wallet_name: String,
        task: JoinHandle<()>,
        cancel_token: CancellationToken,
    ) {
        self.running_for_wallet = Some(wallet_name);
        self.cancel_token = Some(cancel_token);
        self.task = Some(task);
    }
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawns and manages the lifecycle of node and event listener tasks
pub struct NodeLifecycle {
    node_task: JoinHandle<()>,
    event_task: JoinHandle<()>,
}

impl NodeLifecycle {
    pub fn spawn<NodeFut, EventFut>(
        node_future: NodeFut,
        event_future: EventFut,
        cancel_token: CancellationToken,
    ) -> Self
    where
        NodeFut: std::future::Future<Output = ()> + Send + 'static,
        EventFut: std::future::Future<Output = ()> + Send + 'static,
    {
        let node_cancel = cancel_token.clone();
        let event_cancel = cancel_token.clone();

        let node_task = tauri::async_runtime::spawn(async move {
            tokio::select! {
                _ = node_future => {}
                _ = node_cancel.cancelled() => {
                    tracing::info!("Neutrino node stopped");
                }
            }
        });

        let event_task = tauri::async_runtime::spawn(async move {
            tokio::select! {
                _ = event_future => {}
                _ = event_cancel.cancelled() => {
                    tracing::info!("Neutrino client stopped");
                }
            }
        });

        Self {
            node_task,
            event_task,
        }
    }

    /// Wait for cancellation and abort both tasks
    pub async fn wait_for_cancellation(self, cancel_token: CancellationToken) {
        cancel_token.cancelled().await;
        self.node_task.abort();
        self.event_task.abort();
    }
}
