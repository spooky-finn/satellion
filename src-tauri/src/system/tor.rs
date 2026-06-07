use std::sync::Mutex;

use tokio::{
    net::TcpStream,
    time::{Duration, sleep},
};

use crate::config::TorConfig;

pub fn start_blocking(config: &TorConfig) -> Option<TorProcess> {
    let config = config.clone();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(start(&config))
    })
    .join()
    .unwrap()
}

/// Owns a spawned `tor` process and kills it on drop.
/// Wrapping `Child` in `Mutex` makes this `Send + Sync` for use as Tauri state.
pub struct TorProcess {
    _child: Mutex<Option<std::process::Child>>,
}

impl Drop for TorProcess {
    fn drop(&mut self) {
        if let Ok(mut guard) = self._child.lock()
            && let Some(ref mut child) = *guard
        {
            let _ = child.kill();
            tracing::info!("Tor: process stopped");
        }
    }
}

/// Starts Tor if `config.enabled` is true.
///
/// - If the SOCKS5 port is already open (e.g. system Tor daemon), reuses it.
/// - Otherwise spawns `tor --SocksPort <port>` and waits up to 30 s for it to
///   come up.
/// - Returns `None` when disabled or when the `tor` binary is not available
///   (logs an error).
pub async fn start(config: &TorConfig) -> Option<TorProcess> {
    if !config.enabled {
        return None;
    }

    let port = socks_port(&config.socks5_proxy)?;

    if is_open(port).await {
        tracing::info!("Tor: SOCKS5 port {port} already listening — reusing external process");
        return Some(TorProcess {
            _child: Mutex::new(None),
        });
    }

    tracing::info!("Tor: spawning tor on SOCKS5 port {port}");
    let child = match std::process::Command::new("tor")
        .arg("--SocksPort")
        .arg(port.to_string())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Tor: failed to spawn tor binary — {e}");
            tracing::error!("Tor: install tor via your package manager (e.g. `brew install tor`)");
            return None;
        }
    };

    let process = TorProcess {
        _child: Mutex::new(Some(child)),
    };

    for _ in 0..30 {
        sleep(Duration::from_secs(1)).await;
        if is_open(port).await {
            tracing::info!("Tor: ready on port {port}");
            return Some(process);
        }
    }

    tracing::warn!(
        "Tor: process started but port {port} not reachable after 30s — continuing anyway"
    );
    Some(process)
}

async fn is_open(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).await.is_ok()
}

fn socks_port(addr: &str) -> Option<u16> {
    let addr = addr.trim_start_matches("socks5://");
    addr.rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
}
