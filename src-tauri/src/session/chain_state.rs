use zeroize::Zeroize;

use alloy_signer_local::PrivateKeySigner;
use bitcoin::bip32::Xpriv;

pub struct BitcoinSession {
    pub xprv: Xpriv,
}

pub struct EthereumSession {
    pub signer: PrivateKeySigner,
}

pub enum ChainSession {
    Bitcoin(BitcoinSession),
    Ethereum(EthereumSession),
}

impl ChainSession {
    pub fn as_bitcoin(&self) -> Option<&BitcoinSession> {
        match self {
            ChainSession::Bitcoin(config) => Some(config),
            _ => None,
        }
    }

    pub fn as_ethereum(&self) -> Option<&EthereumSession> {
        match self {
            ChainSession::Ethereum(config) => Some(config),
            _ => None,
        }
    }
}

impl From<BitcoinSession> for ChainSession {
    fn from(config: BitcoinSession) -> Self {
        ChainSession::Bitcoin(config)
    }
}

impl From<EthereumSession> for ChainSession {
    fn from(config: EthereumSession) -> Self {
        ChainSession::Ethereum(config)
    }
}

impl Drop for ChainSession {
    fn drop(&mut self) {
        match self {
            ChainSession::Bitcoin(btc) => {
                btc.xprv.private_key.secret_bytes().zeroize();
            }
            ChainSession::Ethereum(eth) => {
                eth.signer.to_bytes().zeroize();
            }
        }
    }
}
