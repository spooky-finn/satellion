use bitcoin::{
    Amount, OutPoint, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    absolute::LockTime,
    bip32::{KeySource, Xpriv},
    key::Secp256k1,
    psbt::Psbt,
    transaction::Version,
};

use crate::chain::btc::{
    account::Account,
    config::BitcoinConfig,
    key_derivation::{Change, KeyDerivationPath, LabeledKeyDerivationPath, Proposal},
    tx_builder::{BuildTxResult, estimate_taproot_vbytes},
    utxo::Utxo,
};

pub struct BuildCpfpParams<'a> {
    pub parent_tx_id: Txid,
    pub target_fee_rate_sat_vb: f64,
    pub config: BitcoinConfig,
    pub account: &'a Account,
    pub xpriv: &'a Xpriv,
}

/// Build a single-output self-send PSBT that spends every UTXO we own which
/// descends from `parent_tx_id`. The output goes to a fresh internal change
/// address so we don't dox the parent's change to anyone watching addresses.
///
/// `target_fee_rate_sat_vb` is applied to the child tx alone. Modern
/// package-aware miners will mine the parent to collect this child fee even
/// if the parent's own rate is below the next-block clearing rate.
pub fn build_cpfp_psbt(p: &BuildCpfpParams) -> Result<BuildTxResult, String> {
    let child_utxos: Vec<&Utxo> = p
        .account
        .utxo_set
        .entries
        .values()
        .filter(|u| u.tx_id == p.parent_tx_id)
        .collect();

    if child_utxos.is_empty() {
        return Err(
            "no spendable outputs from this transaction — cannot CPFP without owning a child output"
                .to_string(),
        );
    }

    let total_input: u64 = child_utxos.iter().map(|u| u.output.value.to_sat()).sum();
    let input_count = child_utxos.len();
    let estimated_vbytes = estimate_taproot_vbytes(input_count, 1);
    let fee: u64 = (estimated_vbytes as f64 * p.target_fee_rate_sat_vb).ceil() as u64;

    if fee >= total_input {
        return Err(format!(
            "fee ({fee} sat) exceeds spendable inputs ({total_input} sat)"
        ));
    }
    let output_value = total_input - fee;

    let change_index = p.account.keychain.next_unused_index(Change::Internal);
    let change_key_path = KeyDerivationPath::new(
        Proposal::Bip86,
        p.config.network(),
        p.account.index,
        Change::Internal,
        change_index,
    );
    let change_child_key = change_key_path
        .derive(p.xpriv)
        .map_err(|e| format!("failed to derive change key: {e}"))?;

    let input: Vec<TxIn> = child_utxos
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: utxo.tx_id,
                vout: utxo.vout,
            },
            script_sig: bitcoin::ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        })
        .collect();

    let output = vec![TxOut {
        value: Amount::from_sat(output_value),
        script_pubkey: change_child_key.taproot_address.script_pubkey(),
    }];

    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output,
    })
    .map_err(|e| format!("Failed to create PSBT: {e}"))?;

    let secp = Secp256k1::new();
    let master_fingerprint = p.xpriv.fingerprint(&secp);
    for (i, utxo) in child_utxos.iter().enumerate() {
        psbt.inputs[i].witness_utxo = Some(utxo.output.clone());
        let child = utxo
            .derivation
            .derive(p.xpriv)
            .map_err(|e| format!("failed to derive child key: {e}"))?;
        let xonly_pubkey = child.keypair.x_only_public_key().0;
        let derivation_path = utxo.derivation.to_path()?;
        let key_source: KeySource = (master_fingerprint, derivation_path);
        psbt.inputs[i]
            .tap_key_origins
            .insert(xonly_pubkey, (vec![], key_source));
        psbt.inputs[i].tap_internal_key = Some(xonly_pubkey);
    }

    Ok(BuildTxResult {
        fee: fee as u32,
        psbt,
        change_key_path: LabeledKeyDerivationPath {
            label: "Change".to_string(),
            path: change_key_path,
        },
    })
}
