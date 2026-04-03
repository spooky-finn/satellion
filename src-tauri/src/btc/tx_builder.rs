use serde::Serialize;
use specta::Type;

use bitcoin::{Address, address::NetworkChecked, hashes::Hash, key::Secp256k1};
use rustywallet_psbt::{KeySource, Psbt, TxOut};

use crate::btc::{
    account::{Account, UtxoSelectionMethod},
    key_derivation::Change,
};

pub struct BuildPsbtParams {
    pub send_value_sat: u64,
    pub recipient: Address<NetworkChecked>,
    pub utxo_selection_method: UtxoSelectionMethod,
}

#[derive(Type, Serialize)]
pub struct BuildTxResult {
    // pub psbt_base64: String,
    // pub fee_sat: String,
    // pub total_input_sat: String,
    // pub total_output_sat: String,
}

/// The PSBT has two outputs:
/// - Output 0: change returned to the wallet's next unused **internal** address
/// - Output 1: recipient payment
pub fn build_psbt(
    params: &BuildPsbtParams,
    account: &Account,
    xpriv: &bitcoin::bip32::Xpriv,
) -> Result<BuildTxResult, String> {
    let utxos = account.select_utxo_for_tx(params.utxo_selection_method.clone());
    if utxos.is_empty() {
        return Err("no utxos selected for transaction".to_string());
    }

    let total_input: u64 = utxos.iter().map(|u| u.output.value.to_sat()).sum();

    // Change = total input - send amount. The remainder is the miner fee.
    let change_value = total_input.checked_sub(params.send_value_sat).ok_or(
        "total input value is less than send value, cannot cover transaction fee".to_string(),
    )?;

    // Create PSBT v2: N inputs, 2 outputs
    let mut psbt = Psbt::new_v2(utxos.len(), 2);

    // Global fields
    let secp = Secp256k1::new();
    let master_fingerprint = xpriv.fingerprint(&secp).to_bytes();

    // ── Populate inputs ──────────────────────────────────────────────────
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
        // Output 0: change
        let change_index = account.unoccupied_address(Change::Internal);
        let change_path =
            Account::new_deriviation_path(account.index, Change::Internal, change_index);
        let change_child_key = change_path
            .derive(xpriv)
            .map_err(|e| format!("failed to derive change key: {e}"))?;
        let change_xonly_pubkey = change_child_key.keypair.x_only_public_key().0.serialize();
        let change_derivation_path: Vec<u32> = change_path.to_slice().to_vec();
        let change_key_source = KeySource::new(master_fingerprint, change_derivation_path);

        psbt.outputs[0].script = Some(change_child_key.address.script_pubkey().to_bytes());
        psbt.outputs[0].amount = Some(change_value);
        psbt.update_output_with_bip32(0, change_xonly_pubkey.to_vec(), change_key_source)
            .map_err(|e| format!("fail to update output bip32: {e}"))?;
    }

    {
        // Output 1: recipient
        psbt.outputs[1].script = Some(params.recipient.script_pubkey().to_bytes());
        psbt.outputs[1].amount = Some(params.send_value_sat);
    }

    Ok(BuildTxResult {})
}
