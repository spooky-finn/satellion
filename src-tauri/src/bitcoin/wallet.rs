use std::str::FromStr;

use bip39::Language;
use bitcoin::{
    Address,
    bip32::{self, DerivationPath, Xpriv},
    key::{Keypair, Secp256k1},
};

pub use bitcoin::network::Network;

pub enum AddressType {
    Receive = 0,
    Change = 1,
}

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

pub fn create_diriviation_path(
    network: Network,
    purpose: AddressType,
    address_index: u32,
) -> DerivationPath {
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let change = match purpose {
        AddressType::Receive => 0,
        AddressType::Change => 1,
    };

    let account = 0;
    let path = format!("m/86'/{coin_type}'/{account}'/{change}/{address_index}");
    DerivationPath::from_str(&path).expect("Derivation path is creation failed")
}

pub fn derive_taproot_address(
    xprv: &Xpriv,
    network: Network,
    purpose: AddressType,
    address_index: u32,
) -> Result<(Keypair, Address), String> {
    let secp = Secp256k1::new();
    let path = create_diriviation_path(network, purpose, address_index);

    // derive child private key
    let keypair = xprv
        .derive_priv(&secp, &path)
        .map_err(|e| format!("Derivation error: {}", e))?
        .to_keypair(&secp);

    // x-only pubkey for taproot
    let (xonly_pk, _parity) = keypair.x_only_public_key();

    // Create taproot address (BIP341 tweak is done automatically by rust-bitcoin)
    let address = Address::p2tr(
        &secp, xonly_pk, None, // no script tree = BIP86 key-path spend
        network,
    );

    Ok((keypair, address))
}
