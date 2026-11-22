use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use rustls::{ClientConfig as RustlsConfig, RootCertStore, pki_types::ServerName};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, split},
    net::TcpStream,
    sync::Mutex,
};
use tokio_rustls::TlsConnector;
use tokio_socks::tcp::Socks5Stream;

use crate::chain::btc::config::BitcoinConfig;

pub const SEEDS: &[(&str, u16)] = &[
    ("electrum.blockstream.info", 50002),
    ("electrum1.bluewallet.io", 443),
    ("electrum2.bluewallet.io", 443),
    ("bitcoin.aranguren.org", 50002),
    ("electrum.bitaroo.net", 50002),
    ("electrum.emzy.de", 50002),
    ("electrum.hodlister.co", 50002),
];

// ── Transport
// ──────────────────────────────────────────────────────────────────

type BoxReader = Box<dyn AsyncRead + Unpin + Send>;
type BoxWriter = Box<dyn AsyncWrite + Unpin + Send>;

struct Conn {
    reader: BufReader<BoxReader>,
    writer: BoxWriter,
}

impl Conn {
    fn from_stream<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(stream: S) -> Self {
        let (r, w) = split(stream);
        Self {
            reader: BufReader::new(Box::new(r)),
            writer: Box::new(w),
        }
    }

    async fn send(&mut self, line: &str) -> Result<(), String> {
        self.writer
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;
        self.writer.flush().await.map_err(|e| format!("flush: {e}"))
    }

    async fn recv_response(&mut self) -> Result<(u64, RpcResp), String> {
        let mut line = String::new();
        loop {
            let bytes = self
                .reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("read: {e}"))?;
            if bytes == 0 {
                return Err("read: connection closed by Electrum server".into());
            }
            if !line.ends_with('\n') {
                return Err("read: incomplete Electrum response".into());
            }
            let response: RpcResp =
                serde_json::from_str(&line).map_err(|e| format!("parse Electrum response: {e}"))?;
            if let Some(id) = response.id {
                return Ok((id, response));
            }
            if response.method.is_none() {
                return Err("Electrum response has no request ID".into());
            }
            // Subscription notifications have a method and no request ID.
            line.clear();
        }
    }
}

// ── Client ─────────────────────────────────────────────────────────────────────

enum Mode {
    Direct(BitcoinConfig),
    Tor(String),
}

pub struct ElectrumClient {
    conn: Mutex<Option<Conn>>,
    mode: Mode,
    request_id: AtomicU64,
}

impl ElectrumClient {
    #[cfg(test)]
    pub(crate) fn from_test_stream<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        stream: S,
    ) -> Self {
        Self {
            conn: Mutex::new(Some(Conn::from_stream(stream))),
            mode: Mode::Tor("unused".into()),
            request_id: AtomicU64::new(1),
        }
    }

    pub fn new(config: BitcoinConfig) -> Self {
        Self {
            conn: Mutex::new(None),
            mode: Mode::Direct(config),
            request_id: AtomicU64::new(1),
        }
    }

    pub fn new_tor(proxy: &str) -> Self {
        Self {
            conn: Mutex::new(None),
            mode: Mode::Tor(proxy.trim_start_matches("socks5://").to_string()),
            request_id: AtomicU64::new(1),
        }
    }

    async fn connect(&self) -> Result<Conn, String> {
        match &self.mode {
            Mode::Direct(cfg) => connect_direct(cfg).await,
            Mode::Tor(proxy) => connect_tor(proxy).await,
        }
    }

    pub async fn request(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.request_inner(method, params),
        )
        .await
        .map_err(|_| format!("Electrum {method} timed out"))?
    }

    async fn request_inner(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let line = build_request_line(id, method, params)?;
        // Own the connection during the exchange. A failure or timeout drops
        // it before another request can read a stale response.
        let mut slot = self.conn.lock().await;
        let mut conn = match slot.take() {
            Some(conn) => conn,
            None => self.connect().await?,
        };
        conn.send(&line).await?;
        let (response_id, response) = conn.recv_response().await?;
        if response_id != id {
            return Err(format!(
                "Electrum response ID mismatch: expected {id}, got {response_id}"
            ));
        }
        *slot = Some(conn);
        response.into_result()
    }

    pub async fn batch(&self, calls: Vec<(&str, Vec<Value>)>) -> Result<Vec<Value>, String> {
        tokio::time::timeout(std::time::Duration::from_secs(60), self.batch_inner(calls))
            .await
            .map_err(|_| "Electrum batch timed out".to_string())?
    }

    async fn batch_inner(&self, calls: Vec<(&str, Vec<Value>)>) -> Result<Vec<Value>, String> {
        if calls.is_empty() {
            return Ok(vec![]);
        }
        let base_id = self
            .request_id
            .fetch_add(calls.len() as u64, Ordering::SeqCst);
        let count = calls.len();
        let lines = calls
            .into_iter()
            .enumerate()
            .map(|(i, (method, params))| build_request_line(base_id + i as u64, method, params))
            .collect::<Result<String, String>>()?;
        let mut slot = self.conn.lock().await;
        let mut conn = match slot.take() {
            Some(conn) => conn,
            None => self.connect().await?,
        };
        conn.send(&lines).await?;

        let mut responses: Vec<Option<Result<Value, String>>> = (0..count).map(|_| None).collect();
        for _ in 0..count {
            let (id, response) = conn.recv_response().await?;
            let Some(index) = id.checked_sub(base_id).filter(|i| *i < count as u64) else {
                return Err(format!("Unexpected Electrum batch response ID {id}"));
            };
            let slot = &mut responses[index as usize];
            if slot.is_some() {
                return Err(format!("Duplicate Electrum batch response ID {id}"));
            }
            *slot = Some(response.into_result());
        }
        *slot = Some(conn);
        responses
            .into_iter()
            .map(|r| r.expect("all responses received"))
            .collect()
    }
}

// ── Connection factories
// ───────────────────────────────────────────────────────

async fn connect_direct(config: &BitcoinConfig) -> Result<Conn, String> {
    if config.regtest {
        let stream = TcpStream::connect("127.0.0.1:50001")
            .await
            .map_err(|e| format!("regtest connect: {e}"))?;
        return Ok(Conn::from_stream(stream));
    }

    let connector = TlsConnector::from(Arc::new(build_tls_config()));

    if let Some(ref server) = config.electrum_server {
        let (host, port) = parse_host_port(server)?;
        return tls_connect(&connector, host, port).await;
    }

    for (host, port) in SEEDS {
        match tls_connect(&connector, host, *port).await {
            Ok(conn) => {
                tracing::info!("Electrum: connected to {host}:{port}");
                return Ok(conn);
            }
            Err(e) => tracing::warn!("Electrum: {host}:{port} failed: {e}"),
        }
    }
    Err("failed to connect to any Electrum server".to_string())
}

async fn connect_tor(proxy: &str) -> Result<Conn, String> {
    let connector = TlsConnector::from(Arc::new(build_tls_config()));
    for (host, port) in SEEDS {
        match socks_tls_connect(&connector, proxy, host, *port).await {
            Ok(conn) => {
                tracing::info!("Electrum over Tor: connected to {host}:{port}");
                return Ok(conn);
            }
            Err(e) => tracing::warn!("Electrum over Tor: {host}:{port} failed: {e}"),
        }
    }
    Err("failed to connect to any Electrum server via Tor".to_string())
}

async fn tls_connect(connector: &TlsConnector, host: &str, port: u16) -> Result<Conn, String> {
    let tcp = TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("TCP: {e}"))?;
    let name: ServerName<'static> = host
        .to_string()
        .try_into()
        .map_err(|e| format!("server name: {e}"))?;
    let tls = connector
        .connect(name, tcp)
        .await
        .map_err(|e| format!("TLS: {e}"))?;
    Ok(Conn::from_stream(tls))
}

async fn socks_tls_connect(
    connector: &TlsConnector,
    proxy: &str,
    host: &str,
    port: u16,
) -> Result<Conn, String> {
    let socks = Socks5Stream::connect(proxy, (host, port))
        .await
        .map_err(|e| format!("SOCKS5: {e}"))?;
    let name: ServerName<'static> = host
        .to_string()
        .try_into()
        .map_err(|e| format!("server name: {e}"))?;
    let tls = connector
        .connect(name, socks)
        .await
        .map_err(|e| format!("TLS: {e}"))?;
    Ok(Conn::from_stream(tls))
}

fn build_tls_config() -> RustlsConfig {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    RustlsConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

// ── JSON-RPC encoding / decoding
// ───────────────────────────────────────────────

#[derive(Serialize)]
struct RpcReq<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: Vec<Value>,
}

#[derive(Deserialize)]
struct RpcResp {
    id: Option<u64>,
    method: Option<String>,
    result: Option<Value>,
    error: Option<RpcErr>,
}

#[derive(Deserialize)]
struct RpcErr {
    message: String,
}

fn build_request_line(id: u64, method: &str, params: Vec<Value>) -> Result<String, String> {
    serde_json::to_string(&RpcReq {
        jsonrpc: "2.0",
        id,
        method,
        params,
    })
    .map(|s| format!("{s}\n"))
    .map_err(|e| e.to_string())
}

impl RpcResp {
    fn into_result(self) -> Result<Value, String> {
        if let Some(err) = self.error {
            return Err(err.message);
        }
        self.result.ok_or_else(|| "empty result".to_string())
    }
}

pub fn parse_host_port(addr: &str) -> Result<(&str, u16), String> {
    addr.rsplit_once(':')
        .and_then(|(host, port)| port.parse::<u16>().ok().map(|p| (host, p)))
        .ok_or_else(|| format!("invalid address '{addr}': expected host:port"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn batch_matches_ids_and_skips_notifications() {
        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let client = ElectrumClient::new_tor("unused");
        *client.conn.lock().await = Some(Conn::from_stream(client_stream));
        let server = tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(server_stream);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            writer.write_all(b"{\"method\":\"blockchain.headers.subscribe\",\"params\":[]}\n{\"id\":2,\"result\":\"second\"}\n{\"id\":1,\"result\":\"first\"}\n").await.unwrap();
        });
        let result = client
            .batch(vec![("one", vec![]), ("two", vec![])])
            .await
            .unwrap();
        assert_eq!(result, vec![Value::from("first"), Value::from("second")]);
        server.await.unwrap();
    }
    #[tokio::test]
    async fn batch_drains_errors_before_next_request() {
        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let client = ElectrumClient::new_tor("unused");
        *client.conn.lock().await = Some(Conn::from_stream(client_stream));
        let server = tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(server_stream);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            writer
                .write_all(
                    b"{\"id\":1,\"error\":{\"message\":\"failed\"}}\n{\"id\":2,\"result\":42}\n",
                )
                .await
                .unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            writer
                .write_all(b"{\"id\":3,\"result\":\"next\"}\n")
                .await
                .unwrap();
        });
        assert_eq!(
            client
                .batch(vec![("one", vec![]), ("two", vec![])])
                .await
                .unwrap_err(),
            "failed"
        );
        assert_eq!(
            client.request("next", vec![]).await.unwrap(),
            Value::from("next")
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn mismatched_reply_discards_connection() {
        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let client = ElectrumClient::new_tor("unused");
        *client.conn.lock().await = Some(Conn::from_stream(client_stream));
        let server = tokio::spawn(async move {
            let (reader, mut writer) = tokio::io::split(server_stream);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            writer
                .write_all(b"{\"id\":99,\"result\":\"wrong\"}\n")
                .await
                .unwrap();
        });
        assert!(
            client
                .request("test", vec![])
                .await
                .unwrap_err()
                .contains("ID mismatch")
        );
        assert!(client.conn.lock().await.is_none());
        server.await.unwrap();
    }
    #[tokio::test]
    async fn cancelled_in_flight_request_drops_connection() {
        let (client_stream, mut server_stream) = tokio::io::duplex(4096);
        let client = ElectrumClient::from_test_stream(client_stream);
        let task = tokio::spawn(async move {
            let mut bytes = [0; 512];
            use tokio::io::AsyncReadExt;
            server_stream.read(&mut bytes).await.unwrap();
            std::future::pending::<()>().await;
        });
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(20),
                client.request_inner("test", vec![])
            )
            .await
            .is_err()
        );
        assert!(client.conn.lock().await.is_none());
        task.abort();
    }
}
