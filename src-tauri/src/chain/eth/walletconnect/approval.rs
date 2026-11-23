use alloy::{
    consensus::{SignableTransaction, TxEnvelope},
    network::TxSignerSync,
    primitives::{B256, hex},
    providers::Provider,
    rpc::types::TransactionRequest,
    signers::{SignerSync, local::PrivateKeySigner},
};
use tokio::sync::Mutex;

use super::{WalletConnectConfig, now, request::Action, state::Approval};
use crate::{
    chain_trait::SecureKey,
    config::Config,
    eth::{provider::checked_http_provider, submission::submit_transaction},
    session::SK,
};

pub(super) async fn approve_request(
    topic: &str,
    id: &str,
    config: &Mutex<Config>,
    sk: &SK,
    confirm: impl AsyncFnOnce(&Approval) -> Result<(), String>,
) -> Result<Option<String>, String> {
    // Configuration changes also acquire these locks in config-then-session order.
    let approval = {
        let config = config.lock().await;
        let settings = WalletConnectConfig::enabled(&config)?;
        let mut sk = sk.lock().await;
        let session = sk.unexpired_session()?;
        let access = session.access().clone();
        session
            .walletconnect
            .claim_approval(topic, id, &settings, access, now())?
    };

    let result = match confirm(&approval).await {
        Ok(()) => approval.execute(config, sk).await,
        Err(error) => Err(error),
    };
    if result.is_err()
        && let Ok(session) = sk.lock().await.unexpired_session()
        && let Some(cancelled) = session.walletconnect.take_confirmation(&approval)
    {
        cancelled.cancel();
    }
    result
}

impl Approval {
    pub(super) fn ensure_active(&self) -> Result<(), String> {
        if self.expires_at <= now() || self.access.is_cancelled() || self.cancelled.is_cancelled() {
            return Err(
                "The wallet locked or the WalletConnect request expired or was cancelled".into(),
            );
        }
        Ok(())
    }

    async fn execute(&self, config: &Mutex<Config>, sk: &SK) -> Result<Option<String>, String> {
        let signature = match &self.prepared.action {
            Action::Transaction(tx) => {
                let hash = self.submit(tx, config, sk).await?;
                return Ok(Some(hash.to_string()));
            }
            Action::Message(message) => {
                self.with_signer(config, sk, |signer| {
                    signer
                        .sign_message_sync(message)
                        .map_err(|_| "Could not sign the message".into())
                })
                .await
            }
            Action::TypedData(hash) => {
                self.with_signer(config, sk, |signer| {
                    signer
                        .sign_hash_sync(hash)
                        .map_err(|_| "Could not sign the typed data".into())
                })
                .await
            }
            Action::SwitchChain => return self.with_signer(config, sk, |_| Ok(None)).await,
        }?;
        Ok(Some(format!("0x{}", hex::encode(signature.as_bytes()))))
    }

    async fn submit(
        &self,
        tx: &TransactionRequest,
        config: &Mutex<Config>,
        sk: &SK,
    ) -> Result<B256, String> {
        let (hash, ()) = submit_transaction(&self.access, Some(&self.cancelled), async {
            self.ensure_active()?;
            let provider = checked_http_provider(&self.url, self.chain).await?;
            let nonce = provider
                .get_transaction_count(self.address)
                .pending()
                .await
                .map_err(|_| "Could not get the pending nonce for this account")?;
            if tx.nonce.is_some_and(|requested| requested != nonce) {
                return Err("The dApp requested a nonce that no longer matches the account".into());
            }
            let mut tx = tx.clone();
            tx.nonce = Some(nonce);
            let mut tx = tx
                .build_1559()
                .map_err(|_| "Could not build the approved Ethereum transaction")?;
            let signature = self
                .with_signer(config, sk, |signer| {
                    signer
                        .sign_transaction_sync(&mut tx)
                        .map_err(|_| "Could not sign the approved transaction".into())
                })
                .await?;
            self.ensure_active()?;
            Ok((provider, TxEnvelope::Eip1559(tx.into_signed(signature)), ()))
        })
        .await?;
        Ok(hash)
    }

    async fn with_signer<T>(
        &self,
        config: &Mutex<Config>,
        sk: &SK,
        sign: impl FnOnce(&PrivateKeySigner) -> Result<T, String>,
    ) -> Result<T, String> {
        let config = config.lock().await;
        let settings = WalletConnectConfig::enabled(&config)?;
        if settings.rpc_url(self.chain)? != self.url {
            return Err("The approved Ethereum RPC changed".into());
        }
        sk.lock()
            .await
            .with_current_session(&self.access, |session| {
                self.ensure_active()?;
                session
                    .walletconnect
                    .take_confirmation(self)
                    .ok_or("The WalletConnect request was cancelled")?;
                let key = session.wallet.eth.prk()?;
                let signer = key.expose();
                if signer.address() != self.address {
                    return Err("The active Ethereum account changed".into());
                }
                sign(signer)
            })
    }
}
