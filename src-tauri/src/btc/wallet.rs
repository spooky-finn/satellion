use std::{collections::HashSet, str::FromStr};

use bip39::Language;
use bip157::ScriptBuf;
pub use bitcoin::network::Network;
use bitcoin::{
    Address,
    bip32::{self, DerivationPath, Xpriv},
    key::{Keypair, Secp256k1},
};

use crate::{config::CONFIG, session::BitcoinSession};

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

#[derive(serde::Serialize, specta::Type)]
pub struct BitcoinUnlock {
    address: String,
    change_address: String,
}

pub fn unlock(mnemonic: &str, passphrase: &str) -> Result<(BitcoinUnlock, BitcoinSession), String> {
    let net = CONFIG.bitcoin.network();
    let bitcoin_xprv = create_private_key(net, mnemonic, passphrase).map_err(|e| e.to_string())?;

    let (_, bitcoin_main_receive_address) =
        derive_taproot_address(&bitcoin_xprv, net, AddressType::Receive, 0)
            .map_err(|e| e.to_string())?;
    let (_, bitcoin_main_change_address) =
        derive_taproot_address(&bitcoin_xprv, net, AddressType::Change, 0)
            .map_err(|e| e.to_string())?;

    Ok((
        BitcoinUnlock {
            address: bitcoin_main_receive_address.to_string(),
            change_address: bitcoin_main_change_address.to_string(),
        },
        BitcoinSession { xprv: bitcoin_xprv },
    ))
}

pub fn derive_scripts_of_interes(xpriv: &Xpriv) -> HashSet<ScriptBuf> {
    let mut scripts_of_interes: HashSet<bip157::ScriptBuf> = HashSet::new();
    let net = CONFIG.bitcoin.network();

    // TODO: remember last index to check
    for i in 0..1000 {
        let (_, bitcoin_main_receive_address) =
            derive_taproot_address(xpriv, net, AddressType::Receive, i)
                .expect("Failed to derive taproot address");
        let scriptbuf = bitcoin_main_receive_address.script_pubkey();
        scripts_of_interes.insert(scriptbuf);
    }

    scripts_of_interes
}
