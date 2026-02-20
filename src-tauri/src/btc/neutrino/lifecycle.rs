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

// /// Spawns and manages the lifecycle of node and event listener tasks
// pub struct NodeLifecycle {
//     pub tasks: Vec<JoinHandle<()>>,
// }

// pub type BoxFutureUnit = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

// impl NodeLifecycle {
//     /// Spawn multiple cancellable tasks
//     pub fn spawn<Fut>(futures: Vec<(Fut, &'static str)>, cancel_token: CancellationToken) -> Self
//     where
//         Fut: std::future::Future<Output = ()> + Send + 'static,
//     {
//         let mut tasks = Vec::with_capacity(futures.len());

//         for (fut, name) in futures {
//             let token = cancel_token.clone();
//             let task = tauri::async_runtime::spawn(async move {
//                 tokio::select! {
//                     _ = fut => {},
//                     _ = token.cancelled() => {
//                         tracing::info!("{} stopped", name);
//                     }
//                 }
//             });
//             tasks.push(task);
//         }

//         Self { tasks }
//     }

//     /// Optionally, wait for all tasks to finish
//     pub async fn join_all(self) {
//         for t in self.tasks {
//             let _ = t.await;
//         }
//     }
// }
