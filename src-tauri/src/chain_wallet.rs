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
