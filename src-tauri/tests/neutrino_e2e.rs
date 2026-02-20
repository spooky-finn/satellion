mod bitcoind;

use std::time::Duration;

use satellion_lib::{
    btc::neutrino::{NeutrinoStarter, NodeStartArgs},
    mnemonic::TEST_MNEMONIC,
    session::{Session, SessionKeeper},
    utils,
    wallet::Wallet,
};
use shush_rs::SecretBox;
use tokio::time::sleep;

use crate::bitcoind::BitcoindHarness;

// type Reactor = nakamoto::net::poll::Reactor<net::TcpStream>;

// const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn neutrino_e2e_connect_and_ready() -> anyhow::Result<()> {
    utils::tracing::init_test("debug");

    let harness = BitcoindHarness::start()?;
    // let deadline = Instant::now() + TEST_TIMEOUT;
    // let config = BitcoinConfig {
    //     regtest: true,
    //     min_peers: 1,
    //     ..Default::default()
    // };

    let wallet_name = "test".to_string();
    let wallet = Wallet::new(
        wallet_name.clone(),
        TEST_MNEMONIC.to_string(),
        SecretBox::new(Box::new("1111".to_string())),
        None,
    )
    .unwrap();

    let sk = SessionKeeper::new(None, None);
    sk.lock().await.set(Session::new(wallet));

    let starter = NeutrinoStarter::new(sk);

    starter
        .request_node_start(NodeStartArgs { birth_height: 0 }, wallet_name)
        .await
        .expect("fail to start node");

    harness.fund_wallet()?;

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

#[test]
fn test_fund_and_send() -> anyhow::Result<()> {
    let harness = BitcoindHarness::start()?;
    harness.fund_wallet()?;

    // let recipient = harness.new_address()?;
    // harness.send_and_confirm(&recipient, 1.0)?;

    let balance = harness.balance()?;
    println!("Balance: {balance}");
    Ok(())
}
