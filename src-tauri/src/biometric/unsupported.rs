//! Stub backend for platforms without a supported biometric flow.

use super::BiometricError;

pub fn is_supported() -> bool {
    false
}

pub fn store(_account: &str, _passphrase: &[u8]) -> Result<(), BiometricError> {
    Err(BiometricError::NotSupported)
}

pub fn read(_account: &str) -> Result<Vec<u8>, BiometricError> {
    Err(BiometricError::NotSupported)
}

pub fn delete(_account: &str) -> Result<(), BiometricError> {
    Ok(())
}

pub fn has(_account: &str) -> Result<bool, BiometricError> {
    Ok(false)
}

pub fn prompt(_reason: &str) -> Result<(), BiometricError> {
    Err(BiometricError::NotSupported)
}
