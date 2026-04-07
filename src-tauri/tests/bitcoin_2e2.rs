mod bitcoind;
mod mocks;

use std::{error::Error, str::FromStr};

use bitcoin::Address;
use satellion_lib::{
    btc::{
        self,
        account::UtxoSelectionMethod,
        config::BitcoinConfig,
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
use shush_rs::SecretBox;

use crate::bitcoind::{BitcoindHarness, parse_scan_result};

#[tokio::test]
async fn bitcon_e2e() -> Result<(), Box<dyn Error>> {
    utils::tracing::init_test("debug");
    let harness = BitcoindHarness::start()?;
    let wallet_name = "test".to_string();
    let sk = SessionKeeper::new(None, None);
    let mut config = Config::new();
    config.btc.regtest = true;

    let sk_clone = sk.clone();
    {
        let wallet = Wallet::new(
            config,
            wallet_name.clone(),
            TEST_MNEMONIC.to_string(),
            SecretBox::new(Box::new("1111".to_string())),
            None,
        )
        .unwrap();
        sk_clone.lock().await.set(Session::new(wallet));
    }

    let balance = harness.balance()?;
    println!("balance before {}", balance.to_btc());

    let account_info = {
        let mut sk = sk.lock().await;
        let wallet = sk.wallet()?;
        let account = wallet.btc.active_account()?;
        let prk = wallet.btc_prk()?;
        let account_info = account.info(&prk, wallet.config.btc.network())?;
        account_info
    };
    harness.fund_wallet()?;
    harness.send_and_confirm(&&account_info.address.to_string(), 1.2)?;

    // Use scantxoutset to find UTXOs for the specific address
    let scan_result = harness.client().call::<serde_json::Value>(
        "scantxoutset",
        &[
            serde_json::json!("start"),
            serde_json::json!([format!("addr({})", account_info.address)]),
        ],
    )?;

    let utxos = parse_scan_result(scan_result).expect("failed to parse scan result");
    println!("Found {} UTXOs for test wallet", utxos.len());

    let utxo = &utxos.get(0).cloned().unwrap();
    for utxo in &utxos {
        println!(
            "UTXO Outpoint: {}, Value: {} BTC",
            utxo.outpoint(),
            utxo.amount
        );
    }
    // TODO: add utxo to the wallet state
    {
        let mut sk = sk.lock().await;
        let wallet = sk.wallet()?;
        let account = wallet.btc.get_mut_active_account()?;
        let network = wallet.config.btc.network();

        let wallet_utxos: Vec<btc::utxo::Utxo> = utxos
            .iter()
            .map(|scanned_utxo| {
                let tx_id = bitcoin::Txid::from_str(&scanned_utxo.txid).expect("invalid txid");
                let vout = scanned_utxo.vout as usize;
                let script_pubkey = bitcoin::ScriptBuf::from_hex(&scanned_utxo.script_pub_key)
                    .expect("invalid script");
                let value = bitcoin::Amount::from_btc(scanned_utxo.amount).expect("invalid amount");
                let output = bitcoin::TxOut {
                    script_pubkey,
                    value,
                };

                // Derivation path: assume external address at index 0 for the main address
                let derivation = btc::key_derivation::KeyDerivationPath::new_bip86(
                    network,
                    account.index,
                    btc::key_derivation::Change::External,
                    0,
                );

                let height = scanned_utxo.height as u32;

                btc::utxo::Utxo {
                    tx_id,
                    vout,
                    output,
                    derivation,
                    height,
                }
            })
            .collect();

        account.set_utxos(wallet_utxos);
    }

    let balance = harness.balance()?;
    println!(
        "Balance {}, utxos count {} \n",
        balance.to_btc(),
        utxos.len()
    );

    let tips = harness.tips()?;
    println!("Tips {:?}", tips);

    // build transaction

    let mut sk = sk.lock().await;
    let wallet = sk.wallet()?;
    let prk = wallet.btc_prk()?;
    let account = wallet.btc.active_account()?;
    let mut config = BitcoinConfig::default();
    config.regtest = true;

    let recipient =
        Address::from_str("bcrt1p04x2uthh0arxzuct6hpetdtg2p7c23yuu855z3srs332ga4k9gasjv0av6")
            .unwrap()
            .assume_checked();

    let ptsb = build_psbt(&BuildPsbtParams {
        send_value_sat: 2000,
        recipient,
        utxo_selection_method: UtxoSelectionMethod::Manual(vec![OutPointDto {
            tx_id: utxo.txid.clone(),
            vout: utxo.vout.to_string(),
        }]),
        miner_fee_vbytes: 100,
        config,
        account,
        xpriv: prk.expose(),
    })
    .expect("failed to create btsp");

    println!("ptsb {:?}", ptsb);

    Ok(())
}
