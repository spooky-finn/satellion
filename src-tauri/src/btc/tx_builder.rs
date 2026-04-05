use serde::Serialize;
use specta::Type;

use bitcoin::{Address, address::NetworkChecked, hashes::Hash, key::Secp256k1};
use rustywallet_psbt::{KeySource, Psbt, TxOut};

use crate::btc::{
    account::{Account, UtxoSelectionMethod},
    config::BitcoinConfig,
    key_derivation::{Change, KeyDerivationPath},
};

const UTXO_DUST_VALUE: u64 = 330;

pub struct BuildPsbtParams {
    pub send_value_sat: u64,
    pub recipient: Address<NetworkChecked>,
    pub utxo_selection_method: UtxoSelectionMethod,
    pub miner_fee_vbytes: u64,
    pub config: BitcoinConfig,
}

#[derive(Type, Serialize)]
pub struct BuildTxResult {}

/// The PSBT has two outputs:
/// - Output 0: change returned to the wallet's next unused **internal** address
/// - Output 1: recipient payment
pub fn build_psbt(
    params: &BuildPsbtParams,
    account: &Account,
    xpriv: &bitcoin::bip32::Xpriv,
) -> Result<BuildTxResult, String> {
    let utxos = account.choose_utxo(params.utxo_selection_method.clone());
    if utxos.is_empty() {
        return Err("no utxos selected for transaction".to_string());
    }
    let input_count = utxos.len();
    let output_count = 2;
    let total_input: u64 = utxos.iter().map(|u| u.output.value.to_sat()).sum();

    // 1. Estimate the fee assuming we WILL have a change output (2 outputs total)
    let estimated_vbytes = estimate_taproot_vbytes(input_count, output_count);
    let required_fee = estimated_vbytes * params.miner_fee_vbytes;

    // 2. Check if the inputs can cover the send amount + fee
    let total_required = params
        .send_value_sat
        .checked_add(required_fee)
        .ok_or("overflow calculating required amount")?;

    let potential_change = total_input
        .checked_sub(total_required)
        .ok_or("insufficient funds to cover send amount and miner fee")?;

    // 3. The Dust Check
    let has_change = potential_change >= UTXO_DUST_VALUE;
    let output_count = if has_change {
        output_count
    } else {
        output_count - 1
    };

    let mut psbt = Psbt::new_v2(input_count, output_count);

    // Global fields
    let secp = Secp256k1::new();
    let master_fingerprint = xpriv.fingerprint(&secp).to_bytes();

    for (i, utxo) in utxos.iter().enumerate() {
        // PSBT v2 required fields
        psbt.inputs[i].previous_txid = Some(utxo.tx_id.to_byte_array());
        psbt.inputs[i].output_index = Some(utxo.vout as u32);
        psbt.inputs[i].sequence = Some(0xFFFFFFFF);

        // Witness UTXO for segwit signing
        psbt.update_input_with_utxo(
            i,
            TxOut {
                value: utxo.output.value.to_sat(),
                script_pubkey: utxo.output.script_pubkey.to_bytes(),
            },
        )
        .map_err(|e| format!("fail to add utxo into psbt: {e}"))?;

        // Derive key and add BIP32 derivation info for hardware wallet compatibility
        let child = utxo
            .derivation
            .derive(xpriv)
            .map_err(|e| format!("failed to derive child key: {e}"))?;
        let xonly_pubkey = child.keypair.x_only_public_key().0.serialize();

        let derivation_path: Vec<u32> = utxo.derivation.to_slice().to_vec();
        let key_source = KeySource::new(master_fingerprint, derivation_path);
        psbt.update_input_with_bip32(i, xonly_pubkey.to_vec(), key_source.clone())
            .map_err(|e| format!("fail to update input bip32: {e}"))?;

        // Taproot internal key (BIP86 key-path spend)
        psbt.inputs[i].tap_internal_key = Some(xonly_pubkey.to_vec());
    }

    {
        // We use a mutable index because the recipient output might be at index 0 or 1
        let mut current_out_idx = 0;

        if has_change {
            // Create the change output
            let change_index = account.unoccupied_address(Change::Internal);
            let change_path = KeyDerivationPath::new_bip86(
                params.config.network(),
                account.index,
                Change::Internal,
                change_index,
            );
            let change_child_key = change_path
                .derive(xpriv)
                .map_err(|e| format!("failed to derive change key: {e}"))?;

            psbt.outputs[current_out_idx].script =
                Some(change_child_key.address.script_pubkey().to_bytes());
            psbt.outputs[current_out_idx].amount = Some(potential_change);

            current_out_idx += 1;
        }

        // Create the recipient output
        psbt.outputs[current_out_idx].script = Some(params.recipient.script_pubkey().to_bytes());
        psbt.outputs[current_out_idx].amount = Some(params.send_value_sat);
    }

    Ok(BuildTxResult {})
}

pub fn estimate_taproot_vbytes(input_count: usize, output_count: usize) -> u64 {
    // 10.5 (overhead) + 57.25 per input + 43 per output
    // We multiply by 4 to work in Weight Units (integers) and divide at the end
    // to avoid floating point math inaccuracies.
    let weight_units = 42 + (229 * input_count as u64) + (172 * output_count as u64);

    // vBytes is Weight / 4, rounded up to the nearest integer
    (weight_units + 3) / 4
}
