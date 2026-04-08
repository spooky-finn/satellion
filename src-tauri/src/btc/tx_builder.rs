use bitcoin::{
    Address, OutPoint, Sequence, Transaction, TxIn, TxOut, Witness,
    absolute::LockTime,
    address::NetworkChecked,
    bip32::{KeySource, Xpriv},
    key::Secp256k1,
    psbt::Psbt,
    transaction::Version,
};

use crate::{
    btc::{
        Prk,
        account::{Account, UtxoSelectionMethod},
        config::BitcoinConfig,
        key_derivation::{Change, KeyDerivationPath},
    },
    chain_trait::SecureKey,
};

const UTXO_DUST_VALUE: u64 = 330;

pub struct BuildPsbtParams<'a> {
    pub send_value_sat: u64,
    pub recipient: Address<NetworkChecked>,
    pub utxo_selection_method: UtxoSelectionMethod,
    pub miner_fee_vbytes: f64,
    pub config: BitcoinConfig,
    pub account: &'a Account,
    pub xpriv: &'a Xpriv,
}

#[derive(Debug)]
pub struct BuildTxResult {
    pub psbt: Psbt,
}

/// The PSBT has two outputs:
/// - Output 0: change returned to the wallet's next unused **internal** address
/// - Output 1: recipient payment
pub fn build_psbt(p: &BuildPsbtParams) -> Result<BuildTxResult, String> {
    let utxos = p.account.choose_utxo(p.utxo_selection_method.clone());
    if utxos.is_empty() {
        return Err("no utxos selected for transaction".to_string());
    }
    let input_count = utxos.len();
    let output_count = 2;
    let total_input: u64 = utxos.iter().map(|u| u.output.value.to_sat()).sum();

    // 1. Estimate the fee assuming we WILL have a change output (2 outputs total)
    let estimated_vbytes = estimate_taproot_vbytes(input_count, output_count);
    let required_fee: u64 = (estimated_vbytes as f64 * p.miner_fee_vbytes).ceil() as u64;

    // 2. Check if the inputs can cover the send amount + fee
    let total_required = p
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

    let input: Vec<TxIn> = utxos
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: utxo.tx_id,
                vout: utxo.vout as u32,
            },
            script_sig: bitcoin::ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        })
        .collect();

    let mut output: Vec<TxOut> = Vec::with_capacity(output_count);

    if has_change {
        // Create the change output
        let change_index = p.account.unoccupied_address(Change::Internal);
        let change_path = KeyDerivationPath::new_bip86(
            p.config.network(),
            p.account.index,
            Change::Internal,
            change_index,
        );
        let change_child_key = change_path
            .derive(p.xpriv)
            .map_err(|e| format!("failed to derive change key: {e}"))?;

        output.push(TxOut {
            value: bitcoin::Amount::from_sat(potential_change),
            script_pubkey: change_child_key.address.script_pubkey(),
        });
    }

    // Create the recipient output
    output.push(TxOut {
        value: bitcoin::Amount::from_sat(p.send_value_sat),
        script_pubkey: p.recipient.script_pubkey(),
    });

    // Create the unsigned transaction
    let tx = bitcoin::Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output,
    };

    // Create PSBT from unsigned transaction
    let mut psbt = Psbt::from_unsigned_tx(tx).map_err(|e| format!("Failed to create PSBT: {e}"))?;

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

    Ok(BuildTxResult { psbt })
}

pub fn sign_psbt(mut psbt: Psbt, prk: &Prk) -> Result<Transaction, Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    // Sign the PSBT - bitcoin crate handles both ECDSA and Schnorr automatically
    // using the xpriv's GetKey implementation which derives keys from the bip32 path
    psbt.sign(prk.expose(), &secp)
        .map_err(|e| format!("Failed to sign PSBT: {:?}", e))?;

    // Build the final transaction with witnesses
    let mut final_tx = psbt.unsigned_tx.clone();

    for input_idx in 0..psbt.inputs.len() {
        let input = &psbt.inputs[input_idx];

        // For Taproot key-path spend
        if let Some(tap_key_sig) = &input.tap_key_sig {
            let mut witness = Witness::new();
            witness.push(tap_key_sig.to_vec());

            if let Some(tx_input) = final_tx.input.get_mut(input_idx) {
                tx_input.witness = witness;
            }
        }
    }

    Ok(final_tx)
}

pub fn estimate_taproot_vbytes(input_count: usize, output_count: usize) -> u64 {
    // 10.5 (overhead) + 57.25 per input + 43 per output
    // We multiply by 4 to work in Weight Units (integers) and divide at the end
    // to avoid floating point math inaccuracies.
    let weight_units = 42 + (229 * input_count as u64) + (172 * output_count as u64);
    // vBytes is Weight / 4, rounded up to the nearest integer
    weight_units.div_ceil(4)
}
