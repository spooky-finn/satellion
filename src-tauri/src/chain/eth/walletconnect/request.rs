use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alloy::{
    primitives::{Address, B256, Bytes, utils::format_ether},
    rpc::types::TransactionRequest,
};
use serde::Deserialize;
use serde_json::Value;
use specta::Type;
use tokio::sync::Mutex;

use super::{
    WalletConnectConfig, WalletConnectReview, active_address, now, transactions, typed_data,
};
use crate::{config::Config, eth::provider::checked_http_provider, session::SK};

pub(super) const METHODS: &[&str] = &[
    "eth_sendTransaction",
    "personal_sign",
    "eth_signTypedData_v4",
    "wallet_switchEthereumChain",
];

const MAX_REQUEST_BYTES: usize = 128 * 1024;
const MAX_REQUEST_LIFETIME: u32 = 7 * 24 * 60 * 60;

#[derive(Deserialize, Type)]
pub struct WalletConnectRequest {
    pub(super) topic: String,
    pub(super) id: String,
    pub(super) chain: u32,
    pub(super) method: String,
    params_json: String,
    pub(super) expires_at: u32,
}

impl WalletConnectRequest {
    fn validate(&self, now: u32) -> Result<(), String> {
        if self.expires_at <= now
            || self.expires_at.saturating_sub(now) > MAX_REQUEST_LIFETIME
            || self.id.parse::<u64>().is_err()
            || self.params_json.len() > MAX_REQUEST_BYTES
        {
            return Err("Invalid or expired WalletConnect request".into());
        }
        Ok(())
    }

    fn time_remaining(&self) -> Duration {
        let expiry = UNIX_EPOCH + Duration::from_secs(self.expires_at.into());
        expiry.duration_since(SystemTime::now()).unwrap_or_default()
    }
}

pub(super) enum Action {
    Transaction(Box<TransactionRequest>),
    Message(Bytes),
    TypedData(B256),
    SwitchChain,
}

pub(super) struct PreparedRequest {
    pub(super) action: Action,
    pub(super) review: WalletConnectReview,
}

pub(super) async fn prepare_request(
    request: &WalletConnectRequest,
    config: &Mutex<Config>,
    sk: &SK,
) -> Result<WalletConnectReview, String> {
    let received_at = now();
    request.validate(received_at)?;
    let (url, preparation, access) = {
        let config = config.lock().await;
        let settings = WalletConnectConfig::enabled(&config)?;
        let url = settings.rpc_url(request.chain)?.to_owned();
        let mut sk = sk.lock().await;
        let session = sk.unexpired_session()?;
        let access = session.access().clone();
        let address = active_address(session)?;
        let preparation = session
            .walletconnect
            .begin_request(request, address, received_at)?;
        (url, preparation, access)
    };

    let prepared = tokio::select! {
        biased;
        _ = access.cancelled() => Err("The wallet locked during request preparation".into()),
        _ = preparation.cancelled.cancelled() => return Err("WalletConnect request was cancelled during preparation".into()),
        _ = tokio::time::sleep(request.time_remaining()) => Err("The WalletConnect request expired".into()),
        result = prepare_action(request, &url, preparation.address, &preparation.chains) => result,
    };
    let config = config.lock().await;
    let settings = WalletConnectConfig::enabled(&config)?;
    let prepared = if settings.rpc_url(request.chain)? != url || request.expires_at <= now() {
        Err("The WalletConnect request expired or its network changed".into())
    } else {
        prepared
    };
    sk.lock()
        .await
        .unexpired_session()?
        .walletconnect
        .finish_request(&preparation, prepared)
}

async fn prepare_action(
    request: &WalletConnectRequest,
    url: &str,
    address: Address,
    chains: &[u32],
) -> Result<PreparedRequest, String> {
    let params: Vec<Value> = serde_json::from_str(&request.params_json)
        .map_err(|_| "Invalid WalletConnect parameters")?;
    let mut review = WalletConnectReview::default();
    let action = match (request.method.as_str(), &params[..]) {
        ("eth_sendTransaction", [transaction]) => {
            let tx = transactions::parse_transaction(transaction, address, request.chain)?;
            let provider = checked_http_provider(url, request.chain).await?;
            transactions::review_transaction(&provider, &tx, &mut review, request.chain, address)
                .await?;
            let (tx, fee) = transactions::estimate_transaction(&provider, tx).await?;
            review.value_eth = Some(format_ether(tx.value.unwrap_or_default()));
            review.fee_eth = Some(format_ether(fee));
            Action::Transaction(Box::new(tx))
        }
        ("personal_sign", [message, from]) => {
            check_signer(from, address)?;
            let message: Bytes = message
                .as_str()
                .ok_or("Expected a hex-encoded message")?
                .parse()
                .map_err(|_| "Invalid hex-encoded message")?;
            review.title = "Sign message".into();
            review.fields.push((
                "Signature scope".into(),
                "Any service that accepts this message can verify the signature.".into(),
            ));
            review.details = Some(typed_data::review_message(&message)?);
            Action::Message(message)
        }
        ("eth_signTypedData_v4", [from, payload]) => {
            check_signer(from, address)?;
            Action::TypedData(typed_data::review_typed_data(
                payload,
                request.chain,
                &mut review,
            )?)
        }
        ("wallet_switchEthereumChain", [params]) => {
            let id = params
                .get("chainId")
                .and_then(Value::as_str)
                .and_then(|id| id.strip_prefix("0x"))
                .and_then(|id| u32::from_str_radix(id, 16).ok())
                .filter(|id| chains.contains(id))
                .ok_or("The requested network is outside the approved session")?;
            review.title = "Switch network".into();
            review.switch_chain = Some(id);
            Action::SwitchChain
        }
        _ => return Err("Unsupported WalletConnect request or parameters".into()),
    };
    Ok(PreparedRequest { action, review })
}

fn check_signer(value: &Value, address: Address) -> Result<(), String> {
    if value
        .as_str()
        .and_then(|value| value.parse::<Address>().ok())
        != Some(address)
    {
        return Err("The requested signer does not match the approved account".into());
    }
    Ok(())
}
