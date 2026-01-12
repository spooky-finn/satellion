/// Unified wallet trait for blockchain-agnostic wallet operations.
pub trait ChainTrait {
    /// The private key type used by this chain for signing.
    type Prk;
    type UnlockContext;
    type UnlockResult;

    /// Unlocks the wallet using the provided private key material.
    async fn unlock(
        &mut self,
        ctx: Self::UnlockContext,
        prk: &Self::Prk,
    ) -> Result<Self::UnlockResult, String>;
}

/// Trait for secure private key handling across different blockchains.
/// Implementations should:
/// - Keep key material encrypted/protected in memory when possible
/// - Zeroize memory when the key is dropped
/// - Minimize the time key material is exposed
pub trait SecureKey {
    /// The underlying key material type. (e.g., Xpriv for Bitcoin, PrivateKeySigner for Ethereum)
    type Material;

    /// Provides access to the underlying key material.
    ///
    /// # Security Considerations
    /// The returned reference should have a minimal lifetime to reduce
    /// the window of potential key exposure.
    fn expose(&self) -> &Self::Material;
}

/// Trait for serialization and deserialization of wallet data.
pub trait Persistable
where
    Self: Sized,
{
    type Serialized: serde::Serialize + for<'de> serde::Deserialize<'de>;

    fn serialize(&self) -> Result<Self::Serialized, String>;

    fn deserialize(data: Self::Serialized) -> Result<Self, String>;
}

/// Trait for tracking assets (tokens, addresses, etc.) across different blockchains.
///
/// This trait provides a unified interface for managing tracked assets in a wallet,
/// enabling generic code that works across different chains. Each blockchain
/// may track different types of assets:
/// - Ethereum: ERC20 tokens
/// - Bitcoin: Derived addresses
/// ```
pub trait AssetTracker<Asset>
where
    Asset: PartialEq,
{
    /// Track a new asset in the wallet.
    ///
    /// This method adds the asset to the wallet's tracking list.
    /// Implementations should validate the asset before adding it.
    fn track(&mut self, asset: Asset) -> Result<(), String>;

    /// Stop tracking an asset by its identifier.
    ///
    /// This method removes an asset from tracking. The identifier is typically
    /// an address string or similar unique identifier.
    fn untrack(&mut self, asset: Asset) -> Result<(), String>;
}
