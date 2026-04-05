mod bitcoind;
mod mocks;

use std::error::Error;

use satellion_lib::{
    config::Config,
    mnemonic::TEST_MNEMONIC,
    session::{Session, SessionKeeper},
    utils,
    wallet::Wallet,
};
use shush_rs::SecretBox;

use crate::bitcoind::BitcoindHarness;

#[tokio::test]
async fn bitcon_e2e() -> Result<(), Box<dyn Error>> {
    utils::tracing::init_test("debug");
    let harness = BitcoindHarness::start()?;
    let wallet_name = "test".to_string();
    let sk = SessionKeeper::new(None, None);
    let mut config = Config::new();
    config.bitcoin.regtest = true;

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

    let address = {
        let mut sk = sk.lock().await;
        let wallet = sk.wallet()?;
        let prk = wallet.btc_prk()?;
        let account_info = wallet.btc.active_account_info(&prk)?;
        account_info.address
    };

    let balance = harness.balance()?;
    println!("balance before {}", balance.to_btc());

    harness.fund_wallet()?;
    harness.send_and_confirm(&address.to_string(), 1.2)?;

    let balance = harness.balance()?;
    println!("balance {}", balance.to_btc());

    let tips = harness.tips()?;
    println!("tips {:?}", tips);

    Ok(())
}
