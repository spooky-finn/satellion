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
    sync::{Mutex, OnceCell},
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

    async fn recv(&mut self) -> Result<String, String> {
        let mut line = String::new();
        let bytes = self
            .reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("read: {e}"))?;
        if bytes == 0 {
            return Err("read: connection closed by Electrum server".to_string());
        }
        Ok(line)
    }
}

// ── Client ─────────────────────────────────────────────────────────────────────

enum Mode {
    Direct(BitcoinConfig),
    Tor(String),
}

pub struct ElectrumClient {
    conn: OnceCell<Mutex<Conn>>,
    mode: Mode,
    request_id: AtomicU64,
}

impl ElectrumClient {
    pub fn new(config: BitcoinConfig) -> Self {
        Self {
            conn: OnceCell::new(),
            mode: Mode::Direct(config),
            request_id: AtomicU64::new(1),
        }
    }

    pub fn new_tor(proxy: &str) -> Self {
        Self {
            conn: OnceCell::new(),
            mode: Mode::Tor(proxy.trim_start_matches("socks5://").to_string()),
            request_id: AtomicU64::new(1),
        }
    }

    async fn get_conn(&self) -> Result<&Mutex<Conn>, String> {
        self.conn
            .get_or_try_init(|| async {
                let conn = match &self.mode {
                    Mode::Direct(cfg) => connect_direct(cfg).await?,
                    Mode::Tor(proxy) => connect_tor(proxy).await?,
                };
                Ok(Mutex::new(conn))
            })
            .await
    }

    pub async fn request(&self, method: &str, params: Vec<Value>) -> Result<Value, String> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let line = build_request_line(id, method, params)?;
        let conn = self.get_conn().await?;
        let mut g = conn.lock().await;
        g.send(&line).await?;
        parse_single(&g.recv().await?)
    }

    pub async fn batch(&self, calls: Vec<(&str, Vec<Value>)>) -> Result<Vec<Value>, String> {
        if calls.is_empty() {
            return Ok(vec![]);
        }
        let base_id = self
            .request_id
            .fetch_add(calls.len() as u64, Ordering::SeqCst);
        let lines = calls
            .iter()
            .enumerate()
            .map(|(i, (method, params))| {
                build_request_line(base_id + i as u64, method, params.clone())
            })
            .collect::<Result<String, String>>()?;
        let conn = self.get_conn().await?;
        let mut g = conn.lock().await;
        g.send(&lines).await?;

        let mut responses = Vec::with_capacity(calls.len());
        for _ in 0..calls.len() {
            responses.push(parse_single(&g.recv().await?)?);
        }
        Ok(responses)
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

fn parse_single(line: &str) -> Result<Value, String> {
    let resp: RpcResp = serde_json::from_str(line).map_err(|e| format!("parse: {e}"))?;
    if let Some(err) = resp.error {
        return Err(err.message);
    }
    resp.result.ok_or_else(|| "empty result".to_string())
}

pub fn parse_host_port(addr: &str) -> Result<(&str, u16), String> {
    addr.rsplit_once(':')
        .and_then(|(host, port)| port.parse::<u16>().ok().map(|p| (host, p)))
        .ok_or_else(|| format!("invalid address '{addr}': expected host:port"))
}
