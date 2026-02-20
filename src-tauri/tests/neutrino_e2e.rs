mod bitcoind;

use std::time::Duration;

use satellion_lib::{
    btc::neutrino::{NeutrinoStarter, NodeStartArgs},
    chain_trait::SecureKey,
    mnemonic::TEST_MNEMONIC,
    session::{Session, SessionKeeper},
    utils,
    wallet::Wallet,
};
use shush_rs::SecretBox;
use tokio::time::sleep;

use crate::bitcoind::BitcoindHarness;

#[tokio::test]
async fn neutrino_e2e_connect_and_ready() -> anyhow::Result<()> {
    utils::tracing::init_test("debug");
    let harness = BitcoindHarness::start()?;
    let wallet_name = "test".to_string();
    let sk = SessionKeeper::new(None, None);

    let sk_clone = sk.clone();
    {
        let wallet = Wallet::new(
            wallet_name.clone(),
            TEST_MNEMONIC.to_string(),
            SecretBox::new(Box::new("1111".to_string())),
            None,
        )
        .unwrap();
        sk_clone.lock().await.set(Session::new(wallet));
    }

    let starter = NeutrinoStarter::new(sk_clone);

    starter
        .request_node_start(NodeStartArgs { birth_height: 0 }, wallet_name)
        .await
        .expect("fail to start node");

    sleep(Duration::from_secs(1)).await;

    let mut sk = sk.lock().await;
    let wallet = sk.wallet_mut()?;

    let prk = wallet.btc_prk()?;
    let derive_path = wallet.btc.main_derive_path();
    let (_, address) = wallet.btc.derive_child(prk.expose(), &derive_path)?;

    let balance = harness.balance()?;
    println!("balance before {}", balance.to_btc());

    harness.fund_wallet()?;
    harness.send_and_confirm(&address, 1.2)?;

    let balance = harness.balance()?;
    println!("balance {}", balance.to_btc());

    let tips = harness.tips()?;
    println!("tips {:?}", tips);
    Ok(())
}
