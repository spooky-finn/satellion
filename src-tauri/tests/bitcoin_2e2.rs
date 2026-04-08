mod bitcoind;
mod mocks;

use std::{error::Error, str::FromStr};

use bitcoin::{Address, Network};
use shush_rs::SecretBox;

use crate::bitcoind::BitcoindHarness;
use satellion_lib::{
    btc::{
        self,
        account::UtxoSelectionMethod,
        key_derivation::{Change, KeyDerivationPath},
        tx_builder::{BuildPsbtParams, build_psbt},
        utxo::OutPointDto,
    },
    chain_trait::SecureKey,
    config::Config,
    mnemonic::TEST_MNEMONIC,
    session::{Session, SessionKeeper},
    utils,
    wallet::Wallet,
};

#[tokio::test]
async fn bitcon_e2e() -> Result<(), Box<dyn Error>> {
    utils::tracing::init_test("debug");
    let local_node = BitcoindHarness::start()?;
    let wallet_name = "test".to_string();
    let sk = SessionKeeper::new(None, None);
    let mut config = Config::new();
    config.btc.regtest = true;

    let wallet = Wallet::new(
        config.clone(),
        wallet_name.clone(),
        TEST_MNEMONIC.to_string(),
        SecretBox::new(Box::new("1111".to_string())),
        None,
    )
    .unwrap();
    sk.lock().await.set(Session::new(wallet));

    let balance = local_node.balance()?;
    println!("balance before {}", balance.to_btc());

    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let account = wallet.btc.active_account()?;
    let prk = wallet.btc_prk()?;
    let account_info = account.info(&prk, wallet.config.btc.network())?;
    let key_derive_path =
        KeyDerivationPath::new_bip86(Network::Regtest, account.index, Change::External, 0);

    local_node.fund_wallet()?;
    local_node.send_and_confirm(&account_info.address.to_string(), 1.2)?;

    let utxos = local_node.scanutxoset(account_info.address, &key_derive_path)?;
    let utxo = utxos.clone()[0].clone();
    println!(
        "UTXO Outpoint: {}, Value: {} BTC",
        utxo.outpoint(),
        utxo.output.value
    );

    // let mut sk = sk.lock().await;
    // let wallet = sk.wallet()?;
    // let prk = wallet.btc_prk()?;
    let account = wallet.btc.get_mut_active_account()?;
    account.set_utxos(utxos.clone());

    let balance = local_node.balance()?;
    println!(
        "Balance {}, utxos count {} \n",
        balance.to_btc(),
        utxos.len()
    );

    let recipient =
        Address::from_str("bcrt1p04x2uthh0arxzuct6hpetdtg2p7c23yuu855z3srs332ga4k9gasjv0av6")
            .unwrap()
            .assume_checked();

    let send_value_sat = 3010;
    let build_res = build_psbt(&BuildPsbtParams {
        send_value_sat,
        recipient: recipient.clone(),
        utxo_selection_method: UtxoSelectionMethod::Manual(vec![OutPointDto {
            tx_id: utxo.tx_id.to_string(),
            vout: utxo.vout.to_string(),
        }]),
        miner_fee_vbytes: 100,
        config: config.btc,
        account,
        xpriv: prk.expose(),
    })
    .expect("failed to create btsp");

    // Sign the PSBT and get the final transaction with witnesses
    let tx = btc::tx_builder::sign_psbt(build_res.psbt, &prk)?;
    // Send the transaction to the local bitcoin node
    let tx_id = local_node.client().send_raw_transaction(&tx)?;
    println!("transaction sent with txid: {:?}", tx_id);
    // Mine a block to confirm the transaction
    local_node.mine_blocks(1)?;

    // Verify the transaction was confirmed by checking it exists
    let tx_result = local_node.client().get_raw_transaction(tx.compute_txid())?;
    println!("transaction retrieved, confirmations: {:?}", tx_result);

    // Verify transaction exists and is confirmed
    assert!(!tx_result.0.is_empty(), "Transaction should be retrievable");

    // Verify that the recipient now has the UTXO after transaction confirmation
    let recipient_scan = local_node.scanutxoset(recipient.to_string(), &key_derive_path)?;

    // Find the UTXO corresponding to the output we sent
    let matching_utxo = recipient_scan
        .iter()
        .find(|utxo| utxo.output.value.to_sat() == send_value_sat);

    assert!(
        matching_utxo.is_some(),
        "Recipient should have the UTXO of {} satoshis after confirmation",
        send_value_sat
    );
    println!(
        "Recipient has UTXO: txid={}, vout={}, amount={} BTC",
        matching_utxo.unwrap().tx_id,
        matching_utxo.unwrap().vout,
        matching_utxo.unwrap().output.value.to_btc()
    );
    Ok(())
}
