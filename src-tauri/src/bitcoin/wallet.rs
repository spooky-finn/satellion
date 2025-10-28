use std::str::FromStr;

use bip39::Language;
use bitcoin::{
    Address,
    bip32::{self, DerivationPath, Xpriv},
    key::{Keypair, Secp256k1},
};

pub use bitcoin::network::Network;

pub fn create_private_key(
    network: Network,
    mnemonic: &str,
    passphrase: &str,
) -> Result<Xpriv, String> {
    let mnemonic = bip39::Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|e| e.to_string())?;
    let seed = mnemonic.to_seed(passphrase);
    let xprv = bip32::Xpriv::new_master(network, &seed).map_err(|e| e.to_string())?;
    Ok(xprv)
}

pub fn derive_main_receive_taproot_address(
    xprv: &Xpriv,
    network: Network,
) -> Result<(Keypair, Address), String> {
    let secp = Secp256k1::new();

    // BIP86 (Taproot): m/86'/0'/0'/0/0
    let path = DerivationPath::from_str("m/86'/0'/0'/0/0")
        .map_err(|e| format!("Invalid derivation path: {}", e))?;

    // derive child private key
    let child_xprv = xprv
        .derive_priv(&secp, &path)
        .map_err(|e| format!("Derivation error: {}", e))?;

    // convert to keypair (needed for taproot tweak)
    let keypair = Keypair::from_secret_key(&secp, &child_xprv.private_key);

    // x-only pubkey for taproot
    let (xonly_pk, _parity) = keypair.x_only_public_key();

    // Create taproot address (BIP341 tweak is done automatically by rust-bitcoin)
    let address = Address::p2tr(
        &secp, xonly_pk, None,    // no script tree = BIP86 key-path spend
        network, // or Network::Testnet
    );

    Ok((keypair, address))
}
