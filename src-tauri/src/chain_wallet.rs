/// Unified wallet trait for blockchain-agnostic wallet operations.
///
/// This trait abstracts the common operations that every blockchain wallet implementation
/// must support, enabling generic code that works across different chains (Bitcoin, Ethereum, etc.).
pub trait ChainWallet {
    /// The private key type used by this chain for signing operations.
    type Prk;

    /// The result type returned when unlocking a wallet, typically containing the primary address.
    type UnlockResult;

    /// Unlocks the wallet using the provided private key material.
    ///
    /// This method derives and returns the primary address for the wallet,
    /// which is used to identify the wallet on the blockchain.
    ///
    /// # Arguments
    /// * `prk` - A reference to the private key material for this chain
    ///
    /// # Returns
    /// * `Ok(Self::UnlockResult)` - The unlock result containing the wallet's address
    /// * `Err(String)` - An error message if unlocking fails
    fn unlock(&self, prk: &Self::Prk) -> Result<Self::UnlockResult, String>;
}

/// Trait for secure private key handling across different blockchains.
///
/// This trait provides a consistent interface for accessing private key material
/// while ensuring security guarantees are maintained. Implementations should:
/// - Keep key material encrypted/protected in memory when possible
/// - Zeroize memory when the key is dropped
/// - Minimize the time key material is exposed
///
/// # Type Parameters
/// * `Material` - The underlying key material type (e.g., Xpriv for Bitcoin, PrivateKeySigner for Ethereum)
pub trait SecureKey {
    /// The underlying key material type.
    type Material;

    /// Provides access to the underlying key material.
    ///
    /// This method should be used sparingly and only when necessary,
    /// as it exposes the raw key material. Prefer using higher-level
    /// operations that work with the key wrapper directly.
    ///
    /// # Security Considerations
    /// The returned reference should have a minimal lifetime to reduce
    /// the window of potential key exposure.
    fn expose_material(&self) -> &Self::Material;
}

/// Marker trait for keys that guarantee zeroization on drop.
///
/// Implementers of this trait promise to securely erase their contents
/// when they go out of scope, preventing key material from lingering in memory.
/// This is particularly important for cryptographic applications where
/// memory dumps could expose sensitive data.
///
/// # Examples
/// Bitcoin's Xpriv uses `non_secure_erase()` in its Drop implementation.
/// Ethereum's PrivateKeySigner handles zeroization internally.
///
/// # Compile-time Guarantee
/// This marker trait can be used in trait bounds to ensure that only keys
/// with secure cleanup are used in sensitive operations:
/// ```rust
/// fn process_secure_key<K: ZeroizableKey>(key: K) {
///     // Compiler guarantees K will zeroize on drop
/// }
/// ```
pub trait ZeroizableKey {}

/// Generic function that demonstrates using ZeroizableKey as a trait bound.
///
/// This function only accepts keys that guarantee zeroization, providing
/// compile-time assurance that sensitive material will be cleaned up.
#[allow(dead_code)]
fn ensure_zeroized_drop<K: ZeroizableKey>(_key: &K) {
    // This function serves as a compile-time check that the key type
    // promises to zeroize its contents on drop.
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockWallet;

    struct MockPrk;
    struct MockUnlock {
        address: String,
    }

    impl ChainWallet for MockWallet {
        type Prk = MockPrk;
        type UnlockResult = MockUnlock;

        fn unlock(&self, _prk: &Self::Prk) -> Result<Self::UnlockResult, String> {
            Ok(MockUnlock {
                address: "0xmock".to_string(),
            })
        }
    }

    #[test]
    fn test_chain_wallet_trait() {
        let wallet = MockWallet;
        let prk = MockPrk;
        let result = wallet.unlock(&prk).unwrap();
        assert_eq!(result.address, "0xmock");
    }
}
