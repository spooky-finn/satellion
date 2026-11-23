use std::future::pending;

use alloy::{consensus::TxEnvelope, primitives::B256};
use alloy_provider::DynProvider;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::provider::broadcast_signed;
use crate::session::SessionAccess;

static SUBMISSION: Mutex<()> = Mutex::const_new(());

enum Revoked {
    WalletLocked,
    RequestCancelled,
}

impl Revoked {
    fn before_signing(&self) -> &'static str {
        match self {
            Self::WalletLocked => "The wallet locked before signing",
            Self::RequestCancelled => "WalletConnect request was cancelled before signing",
        }
    }

    fn after_signing(&self, hash: B256) -> String {
        let reason = match self {
            Self::WalletLocked => "Wallet locked",
            Self::RequestCancelled => "WalletConnect request was cancelled",
        };
        format!("{reason}. Check transaction {hash} before retrying.")
    }
}

/// Serialize the nonce check, local signing, and broadcast for Ethereum sends.
/// The signing future starts under the submission lock and can return context
/// for the caller, such as the account to record in transaction history.
pub(super) async fn submit_transaction<T>(
    access: &SessionAccess,
    request: Option<&CancellationToken>,
    sign: impl Future<Output = Result<(DynProvider, TxEnvelope, T), String>>,
) -> Result<(B256, T), String> {
    let _submission = tokio::select! {
        biased;
        reason = revoked(access, request) => return Err(reason.before_signing().into()),
        lock = SUBMISSION.lock() => lock,
    };
    let (provider, signed, context) = tokio::select! {
        biased;
        reason = revoked(access, request) => return Err(reason.before_signing().into()),
        result = sign => result?,
    };
    let hash = *signed.tx_hash();
    let hash = tokio::select! {
        biased;
        reason = revoked(access, request) => return Err(reason.after_signing(hash)),
        result = broadcast_signed(&provider, signed) => result?,
    };
    Ok((hash, context))
}

async fn revoked(access: &SessionAccess, request: Option<&CancellationToken>) -> Revoked {
    let cancelled = async {
        match request {
            Some(request) => request.cancelled().await,
            None => pending().await,
        }
    };
    tokio::select! {
        biased;
        _ = access.cancelled() => Revoked::WalletLocked,
        _ = cancelled => Revoked::RequestCancelled,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy::{
        consensus::{SignableTransaction, TxEip1559},
        network::TxSignerSync,
        providers::{Provider, RootProvider},
        signers::local::PrivateKeySigner,
    };

    use super::*;

    #[tokio::test]
    async fn disconnected_grant_leaves_submission_queue_without_signing() {
        let _busy = SUBMISSION.lock().await;
        let access = SessionAccess::default();
        let grant = CancellationToken::new();
        grant.cancel();
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            submit_transaction::<()>(&access, Some(&grant), async {
                panic!("disconnected grant reached signing");
            }),
        )
        .await
        .expect("revoked submission should leave the queue");
        assert_eq!(
            result.unwrap_err(),
            "WalletConnect request was cancelled before signing"
        );
    }

    #[tokio::test]
    async fn disconnection_after_signing_reports_the_signed_hash() {
        let access = SessionAccess::default();
        let grant = CancellationToken::new();
        let mut tx = TxEip1559::default();
        let signature = PrivateKeySigner::random()
            .sign_transaction_sync(&mut tx)
            .unwrap();
        let signed = TxEnvelope::Eip1559(tx.into_signed(signature));
        let expected = *signed.tx_hash();
        let error = submit_transaction(&access, Some(&grant), async {
            let provider = RootProvider::new_http("http://127.0.0.1:1".parse().unwrap()).erased();
            grant.cancel();
            Ok((provider, signed, ()))
        })
        .await
        .unwrap_err();
        assert_eq!(
            error,
            format!(
                "WalletConnect request was cancelled. Check transaction {expected} before retrying."
            )
        );
    }
}
