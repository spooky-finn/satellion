//! Touch ID / device passcode prompt via LocalAuthentication. Private to
//! the `biometric` module.

use std::sync::{Arc, Condvar, Mutex};

use block2::StackBlock;
use objc2::{rc::Retained, runtime::Bool};
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LAContext, LAPolicy};

use super::BiometricError;

pub fn is_supported() -> bool {
    unsafe {
        let ctx = LAContext::new();
        ctx.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
            .is_ok()
    }
}

/// Blocks the calling thread on the system biometric prompt. Callers
/// running inside an async runtime must invoke this from a blocking task
/// (e.g. `tokio::task::spawn_blocking`).
pub fn prompt(reason: &str) -> Result<(), BiometricError> {
    let ctx = unsafe { LAContext::new() };
    let reason_ns = NSString::from_str(reason);

    let pair: Arc<(Mutex<Option<Result<(), BiometricError>>>, Condvar)> =
        Arc::new((Mutex::new(None), Condvar::new()));
    let pair_cb = Arc::clone(&pair);

    let block = StackBlock::new(move |success: Bool, error: *mut NSError| {
        let result = if success.as_bool() {
            Ok(())
        } else {
            let msg = if !error.is_null() {
                unsafe { Retained::retain(error) }
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_else(|| "Authentication failed".to_string())
            } else {
                "Authentication failed".to_string()
            };
            if msg.to_lowercase().contains("cancel") {
                Err(BiometricError::UserCancelled)
            } else {
                Err(BiometricError::Backend(msg))
            }
        };
        let (lock, cvar) = &*pair_cb;
        let mut guard = lock.lock().unwrap();
        if guard.is_none() {
            *guard = Some(result);
            cvar.notify_one();
        }
    })
    .copy();

    unsafe {
        ctx.evaluatePolicy_localizedReason_reply(
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
            &reason_ns,
            &block,
        );
    }

    let (lock, cvar) = &*pair;
    let mut guard = lock.lock().unwrap();
    while guard.is_none() {
        guard = cvar.wait(guard).unwrap();
    }
    guard.take().unwrap()
}
