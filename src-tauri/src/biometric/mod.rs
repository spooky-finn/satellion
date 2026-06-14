//! Optional biometric unlock for wallets.
//!
//! Callers operate strictly on domain-level operations (`enable`, `disable`,
//! `prompt_unlock`, `migrate`, `forget`, `is_supported`, `is_enabled`). The
//! choice of OS backend (macOS Keychain + LocalAuthentication today) and
//! every FFI detail lives behind this module.
//!
//! Passphrases never cross the module boundary as a plain `String` — they
//! travel as [`Passphrase`] (a `SecretBox<String>` from `shush-rs`) so the
//! existing zeroize discipline is preserved.

mod error;
pub use error::BiometricError;

#[cfg(target_os = "macos")]
mod keychain;
#[cfg(target_os = "macos")]
mod prompter;
#[cfg(not(target_os = "macos"))]
mod unsupported;

#[cfg(target_os = "macos")]
use keychain as storage;
#[cfg(target_os = "macos")]
use prompter as auth;
use shush_rs::{ExposeSecret, SecretBox};
#[cfg(not(target_os = "macos"))]
use unsupported as auth;
#[cfg(not(target_os = "macos"))]
use unsupported as storage;

/// Wallet passphrase as held by callers of this module.
pub type Passphrase = SecretBox<String>;

/// True when the current device can perform biometric (or device-passcode)
/// authentication.
pub fn is_supported() -> bool {
    auth::is_supported()
}

/// True when the user has enrolled `wallet_name` for biometric unlock.
pub fn is_enabled(wallet_name: &str) -> Result<bool, BiometricError> {
    if !is_supported() {
        return Ok(false);
    }
    storage::has(wallet_name)
}

/// Persist `passphrase` so future calls to [`prompt_unlock`] can recover it
/// after biometric authentication. Overwrites any prior entry.
pub fn enable(wallet_name: &str, passphrase: &Passphrase) -> Result<(), BiometricError> {
    if !is_supported() {
        return Err(BiometricError::NotSupported);
    }
    storage::store(wallet_name, passphrase.expose_secret().as_bytes())
}

/// Remove the stored passphrase for `wallet_name`. No-op if it doesn't exist.
pub fn disable(wallet_name: &str) -> Result<(), BiometricError> {
    storage::delete(wallet_name)
}

/// Best-effort cleanup used when a wallet is deleted from disk. Errors are
/// swallowed because the wallet is already gone.
pub fn forget(wallet_name: &str) {
    let _ = storage::delete(wallet_name);
}

/// Move the stored passphrase from `from` to `to` after a wallet rename.
/// No-op if the wallet wasn't enrolled. The caller provides the passphrase
/// (already in memory thanks to the active session), so no extra biometric
/// prompt is needed.
pub fn migrate(from: &str, to: &str, passphrase: &Passphrase) -> Result<(), BiometricError> {
    if from == to {
        return Ok(());
    }
    if !matches!(storage::has(from), Ok(true)) {
        return Ok(());
    }
    storage::store(to, passphrase.expose_secret().as_bytes())?;
    let _ = storage::delete(from);
    Ok(())
}

/// Run the biometric prompt and, on success, recover the stored passphrase
/// for `wallet_name`. The prompt + read pair is intentionally fused so
/// callers can never grab the secret without first proving user presence.
pub async fn prompt_unlock(wallet_name: &str) -> Result<Passphrase, BiometricError> {
    if !is_supported() {
        return Err(BiometricError::NotSupported);
    }
    let wallet = wallet_name.to_string();
    tokio::task::spawn_blocking(move || -> Result<Passphrase, BiometricError> {
        auth::prompt(&format!("unlock wallet {}", wallet))?;
        let mut bytes = storage::read(&wallet)?;
        let result = String::from_utf8(bytes.clone())
            .map(|s| SecretBox::new(Box::new(s)))
            .map_err(|_| BiometricError::CorruptedSecret);
        zeroize::Zeroize::zeroize(&mut bytes);
        result
    })
    .await
    .map_err(|e| BiometricError::Backend(e.to_string()))?
}
