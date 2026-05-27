use std::cmp::max;

use bitcoin::{
    Address, OutPoint, Sequence, Transaction, TxIn, TxOut, Witness,
    absolute::LockTime,
    address::NetworkChecked,
    bip32::{KeySource, Xpriv},
    key::Secp256k1,
    psbt::Psbt,
    transaction::Version,
};
use miniscript::psbt::PsbtExt;

use crate::{
    chain::btc::{
        Prk,
        account::{Account, UtxoSelectionStrategy},
        config::BitcoinConfig,
        key_derivation::{Change, KeyDerivationPath, LabeledKeyDerivationPath, Proposal},
    },
    chain_trait::SecureKey,
};

const UTXO_DUST_VALUE: u64 = 330;

pub struct BuildPsbtParams<'a> {
    pub send_value_sat: u64,
    pub recipient: Address<NetworkChecked>,
    pub utxo_selection_method: UtxoSelectionStrategy,
    pub miner_fee_vbytes: f64,
    pub config: BitcoinConfig,
    pub account: &'a Account,
    pub xpriv: &'a Xpriv,
}

#[derive(Debug)]
pub struct BuildTxResult {
    pub psbt: Psbt,
    pub change_key_path: LabeledKeyDerivationPath,
}

const MIN_RELAY_FEE: u64 = 16;

/// The PSBT has two outputs:
/// - Output 0: change returned to the wallet's next unused **internal** address
/// - Output 1: recipient payment
pub fn build_psbt(p: &BuildPsbtParams) -> Result<BuildTxResult, String> {
    let utxos = p.account.utxo_set.select(p.utxo_selection_method.clone());
    if utxos.is_empty() {
        return Err("no utxos selected for transaction".to_string());
    }
    let input_count = utxos.len();
    let total_input: u64 = utxos.iter().map(|u| u.output.value.to_sat()).sum();

    let amounts = resolve_amounts(
        total_input,
        input_count,
        p.send_value_sat,
        p.miner_fee_vbytes,
    )?;
    let output_count = if amounts.has_change { 2 } else { 1 };

    let input: Vec<TxIn> = utxos
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: utxo.tx_id,
                vout: utxo.vout,
            },
            script_sig: bitcoin::ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        })
        .collect();

    let mut output: Vec<TxOut> = Vec::with_capacity(output_count);

    let change_index = p.account.keychain.next_unused_index(Change::Internal);
    let change_key_path = KeyDerivationPath::new(
        Proposal::Bip86,
        p.config.network(),
        p.account.index,
        Change::Internal,
        change_index,
    );

    if amounts.has_change {
        let change_child_key = change_key_path
            .derive(p.xpriv)
            .map_err(|e| format!("failed to derive change key: {e}"))?;
        output.push(TxOut {
            value: bitcoin::Amount::from_sat(amounts.change_value_sat),
            script_pubkey: change_child_key.taproot_address.script_pubkey(),
        });
    }

    // Create the recipient output
    output.push(TxOut {
        value: bitcoin::Amount::from_sat(amounts.send_value_sat),
        script_pubkey: p.recipient.script_pubkey(),
    });

    // Create PSBT from unsigned transaction
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output,
    })
    .map_err(|e| format!("Failed to create PSBT: {e}"))?;

    // Add witness UTXOs and BIP32 derivation info to inputs
    let secp = Secp256k1::new();
    let master_fingerprint = p.xpriv.fingerprint(&secp);

    for (i, utxo) in utxos.iter().enumerate() {
        // Add witness UTXO
        psbt.inputs[i].witness_utxo = Some(utxo.output.clone());

        // Derive key and add BIP32 derivation info
        let child = utxo
            .derivation
            .derive(p.xpriv)
            .map_err(|e| format!("failed to derive child key: {e}"))?;
        let xonly_pubkey = child.keypair.x_only_public_key().0;

        let derivation_path = utxo.derivation.to_path()?;
        let key_source: KeySource = (master_fingerprint, derivation_path);

        // Add to tap_key_origins for Taproot inputs
        psbt.inputs[i]
            .tap_key_origins
            .insert(xonly_pubkey, (vec![], key_source));

        // Taproot internal key (BIP86 key-path spend)
        psbt.inputs[i].tap_internal_key = Some(xonly_pubkey);
    }

    Ok(BuildTxResult {
        psbt,
        change_key_path: LabeledKeyDerivationPath {
            label: "Change".to_string(),
            path: change_key_path,
        },
    })
}

pub fn sign_psbt(mut psbt: Psbt, prk: &Prk) -> Result<Transaction, String> {
    let secp = Secp256k1::new();

    psbt.sign(prk.expose(), &secp)
        .map_err(|e| format!("Failed to sign PSBT: {:?}", e))?;

    psbt.finalize_mut(&secp)
        .map_err(|_| "fail to finalize tx".to_string())?;

    psbt.extract_tx().map_err(|e| e.to_string())
}

struct ResolvedAmounts {
    send_value_sat: u64,
    change_value_sat: u64,
    has_change: bool,
}

/// Resolve the recipient and change amounts after accounting for the miner fee.
///
/// When `requested_send_value` equals `total_input`, the transaction sweeps the
/// selected UTXOs: the fee is subtracted from the send amount and no change
/// output is produced. Otherwise the fee is added on top of the requested send
/// value, and any leftover above the dust threshold becomes change.
fn resolve_amounts(
    total_input: u64,
    input_count: usize,
    requested_send_value: u64,
    miner_fee_vbytes: f64,
) -> Result<ResolvedAmounts, String> {
    let is_sweep = requested_send_value == total_input;

    let assumed_outputs = if is_sweep { 1 } else { 2 };
    let estimated_vbytes = estimate_taproot_vbytes(input_count, assumed_outputs);
    let required_fee: u64 = (estimated_vbytes as f64 * miner_fee_vbytes).ceil() as u64;
    let fee = max(required_fee, MIN_RELAY_FEE);

    let (send_value_sat, potential_change) = if is_sweep {
        let send = total_input
            .checked_sub(fee)
            .ok_or("insufficient funds to cover miner fee")?;
        (send, 0)
    } else {
        let total_required = requested_send_value
            .checked_add(fee)
            .ok_or("overflow calculating required amount")?;
        let change = total_input
            .checked_sub(total_required)
            .ok_or("insufficient funds to cover send amount and miner fee")?;
        (requested_send_value, change)
    };

    let has_change = potential_change >= UTXO_DUST_VALUE;
    let change_value_sat = if has_change { potential_change } else { 0 };

    Ok(ResolvedAmounts {
        send_value_sat,
        change_value_sat,
        has_change,
    })
}

pub fn estimate_taproot_vbytes(input_count: usize, output_count: usize) -> u64 {
    // 10.5 (overhead) + 57.25 per input + 43 per output
    // We multiply by 4 to work in Weight Units (integers) and divide at the end
    // to avoid floating point math inaccuracies.
    let weight_units = 42 + (229 * input_count as u64) + (172 * output_count as u64);
    // vBytes is Weight / 4, rounded up to the nearest integer
    weight_units.div_ceil(4)
}
