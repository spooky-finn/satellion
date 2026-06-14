use std::fmt;

#[derive(Debug)]
pub enum BiometricError {
    NotSupported,
    NotConfigured,
    UserCancelled,
    CorruptedSecret,
    Backend(String),
}

impl fmt::Display for BiometricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSupported => {
                write!(f, "Biometric unlock is not supported on this platform")
            }
            Self::NotConfigured => {
                write!(f, "Biometric unlock is not configured for this wallet")
            }
            Self::UserCancelled => write!(f, "Biometric authentication cancelled"),
            Self::CorruptedSecret => write!(f, "Stored passphrase is corrupted"),
            Self::Backend(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BiometricError {}

impl From<BiometricError> for String {
    fn from(e: BiometricError) -> String {
        e.to_string()
    }
}
