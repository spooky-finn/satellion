use alloy_signer_local::PrivateKeySigner;
use bitcoin::bip32::Xpriv;

#[derive(Clone)]
pub struct BitcoinSession {
    pub xprv: Xpriv,
}

#[derive(Clone)]
pub struct EthereumSession {
    pub signer: PrivateKeySigner,
}

#[derive(Clone)]
pub enum ChainData {
    Bitcoin(BitcoinSession),
    Ethereum(EthereumSession),
}

impl ChainData {
    pub fn as_bitcoin(&self) -> Option<&BitcoinSession> {
        match self {
            ChainData::Bitcoin(config) => Some(config),
            _ => None,
        }
    }

    pub fn as_ethereum(&self) -> Option<&EthereumSession> {
        match self {
            ChainData::Ethereum(config) => Some(config),
            _ => None,
        }
    }
}

impl From<BitcoinSession> for ChainData {
    fn from(config: BitcoinSession) -> Self {
        ChainData::Bitcoin(config)
    }
}

impl From<EthereumSession> for ChainData {
    fn from(config: EthereumSession) -> Self {
        ChainData::Ethereum(config)
    }
}
