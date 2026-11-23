use std::collections::HashMap;

use alloy::primitives::Address;
use serde::Deserialize;
use specta::Type;
use tokio_util::sync::CancellationToken;

use super::{
    WalletConnectConfig, WalletConnectReview,
    request::{METHODS, PreparedRequest, WalletConnectRequest},
};
use crate::session::SessionAccess;

const MAX_GRANTS: usize = 16;
const MAX_REQUESTS_PER_GRANT: usize = 128;

#[derive(Default)]
pub(crate) struct WalletConnectState {
    grants: HashMap<String, Grant>,
    pending: Option<PendingRequest>,
}

#[derive(Deserialize, Type)]
pub struct WalletConnectGrant {
    topic: String,
    address: String,
    chains: Vec<WalletConnectChainGrant>,
}

#[derive(Deserialize, Type)]
struct WalletConnectChainGrant {
    id: u32,
    methods: Vec<String>,
}

impl WalletConnectGrant {
    pub(super) fn validate(&self, settings: &WalletConnectConfig<'_>) -> Result<(), String> {
        if self.topic.len() != 64
            || !self.topic.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.chains.is_empty()
        {
            return Err("Invalid WalletConnect session permissions".into());
        }
        for chain in &self.chains {
            settings.rpc_url(chain.id)?;
            if chain
                .methods
                .iter()
                .any(|method| !METHODS.contains(&method.as_str()))
            {
                return Err("Invalid WalletConnect session methods".into());
            }
        }
        Ok(())
    }
}

struct Grant {
    address: Address,
    chains: Vec<WalletConnectChainGrant>,
    requests: HashMap<String, u32>,
    cancelled: CancellationToken,
}

impl Drop for Grant {
    fn drop(&mut self) {
        self.cancelled.cancel();
    }
}

/// Distinguishes requests even when a dApp reuses a JSON-RPC ID after expiry.
#[derive(Clone, PartialEq, Eq)]
struct RequestKey {
    topic: String,
    id: String,
    instance: u128,
}

impl RequestKey {
    fn matches(&self, topic: &str, id: &str) -> bool {
        self.topic == topic && self.id == id
    }
}

struct PendingRequest {
    key: RequestKey,
    address: Address,
    chain: u32,
    expires_at: u32,
    cancelled: CancellationToken,
    phase: RequestPhase,
}

enum RequestPhase {
    Preparing,
    Ready(Box<PreparedRequest>),
    Confirming,
}

pub(super) struct Preparation {
    key: RequestKey,
    pub(super) address: Address,
    pub(super) chains: Vec<u32>,
    pub(super) cancelled: CancellationToken,
}

pub(super) struct Approval {
    key: RequestKey,
    pub(super) address: Address,
    pub(super) chain: u32,
    pub(super) expires_at: u32,
    pub(super) url: String,
    pub(super) prepared: PreparedRequest,
    pub(super) access: SessionAccess,
    pub(super) cancelled: CancellationToken,
}

impl WalletConnectState {
    pub(super) fn authorize(
        &mut self,
        grant: WalletConnectGrant,
        address: Address,
    ) -> Result<(), String> {
        if grant.address.parse::<Address>().ok() != Some(address) {
            return Err("The active Ethereum account changed".into());
        }
        if self.grants.len() >= MAX_GRANTS || self.grants.contains_key(&grant.topic) {
            return Err("Disconnect a WalletConnect session before connecting another".into());
        }
        self.grants.insert(
            grant.topic,
            Grant {
                address,
                chains: grant.chains,
                requests: HashMap::new(),
                cancelled: CancellationToken::new(),
            },
        );
        Ok(())
    }

    pub(super) fn disconnect(&mut self, topic: &str) {
        self.grants.remove(topic);
        self.pending.take_if(|pending| pending.key.topic == topic);
    }

    pub(super) fn begin_request(
        &mut self,
        request: &WalletConnectRequest,
        address: Address,
        now: u32,
    ) -> Result<Preparation, String> {
        self.cancel_pending_if(|pending| pending.expires_at <= now);
        if self.pending.is_some() {
            return Err("Another WalletConnect request is awaiting review".into());
        }
        let grant = self
            .grants
            .get_mut(&request.topic)
            .ok_or("WalletConnect session is disconnected")?;
        if grant.address != address
            || !grant
                .chains
                .iter()
                .any(|chain| chain.id == request.chain && chain.methods.contains(&request.method))
        {
            return Err("This account, network or method is outside the approved session".into());
        }
        grant.requests.retain(|_, expiry| *expiry > now);
        if grant.requests.len() >= MAX_REQUESTS_PER_GRANT
            || grant.requests.contains_key(&request.id)
        {
            return Err("Duplicate request or session request limit reached".into());
        }
        grant
            .requests
            .insert(request.id.clone(), request.expires_at);
        let key = RequestKey {
            topic: request.topic.clone(),
            id: request.id.clone(),
            instance: rand::random(),
        };
        let cancelled = grant.cancelled.child_token();
        self.pending = Some(PendingRequest {
            key: key.clone(),
            address,
            chain: request.chain,
            expires_at: request.expires_at,
            cancelled: cancelled.clone(),
            phase: RequestPhase::Preparing,
        });
        Ok(Preparation {
            key,
            address,
            chains: grant.chains.iter().map(|chain| chain.id).collect(),
            cancelled,
        })
    }

    pub(super) fn finish_request(
        &mut self,
        preparation: &Preparation,
        result: Result<PreparedRequest, String>,
    ) -> Result<WalletConnectReview, String> {
        let pending = self
            .pending
            .as_mut()
            .filter(|pending| {
                pending.key == preparation.key && matches!(pending.phase, RequestPhase::Preparing)
            })
            .ok_or("WalletConnect request was cancelled during preparation")?;
        match result {
            Ok(prepared) => {
                let review = prepared.review.clone();
                pending.phase = RequestPhase::Ready(Box::new(prepared));
                Ok(review)
            }
            Err(error) => {
                pending.cancelled.cancel();
                self.pending = None;
                Err(error)
            }
        }
    }

    pub(super) fn reject(&mut self, topic: &str, id: &str) {
        self.cancel_pending_if(|pending| pending.key.matches(topic, id));
    }

    fn cancel_pending_if(&mut self, matches: impl FnOnce(&PendingRequest) -> bool) {
        if let Some(pending) = self.pending.take_if(|pending| matches(pending)) {
            pending.cancelled.cancel();
        }
    }

    pub(super) fn claim_approval(
        &mut self,
        topic: &str,
        id: &str,
        settings: &WalletConnectConfig<'_>,
        access: SessionAccess,
        now: u32,
    ) -> Result<Approval, String> {
        let pending = self
            .pending
            .as_mut()
            .filter(|pending| {
                pending.key.matches(topic, id) && !matches!(pending.phase, RequestPhase::Confirming)
            })
            .ok_or("No matching WalletConnect request to approve")?;
        if pending.cancelled.is_cancelled() || pending.expires_at <= now {
            return Err("WalletConnect request expired or disconnected".into());
        }
        let url = settings.rpc_url(pending.chain)?.to_owned();
        let prepared = match std::mem::replace(&mut pending.phase, RequestPhase::Confirming) {
            RequestPhase::Ready(prepared) => *prepared,
            previous => {
                pending.phase = previous;
                return Err("The WalletConnect request is still being prepared".into());
            }
        };
        Ok(Approval {
            key: pending.key.clone(),
            address: pending.address,
            chain: pending.chain,
            expires_at: pending.expires_at,
            url,
            prepared,
            access,
            cancelled: pending.cancelled.clone(),
        })
    }

    /// Removes the exact approval. On success the token remains live for
    /// broadcast.
    pub(super) fn take_confirmation(&mut self, approval: &Approval) -> Option<CancellationToken> {
        self.pending
            .take_if(|pending| {
                pending.key == approval.key && matches!(pending.phase, RequestPhase::Confirming)
            })
            .map(|pending| pending.cancelled)
    }
}
