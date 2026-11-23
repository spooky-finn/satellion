use std::time::Duration;

use alloy::{
    primitives::{Address, Bytes, Signature, U256, address},
    providers::Provider,
};
use alloy_node_bindings::Anvil;
use alloy_provider::ext::AnvilApi;
use serde_json::json;
use tokio::sync::Mutex;

use super::{
    WalletConnectConfig, WalletConnectConfigChange, WalletConnectGrant, WalletConnectRequest,
    WalletConnectReview, WalletConnectState, active_address,
    approval::approve_request,
    now,
    request::{Action, PreparedRequest, prepare_request},
    state::Approval,
    transactions::parse_transaction,
};
use crate::{
    config::Config,
    eth::{
        TxBuilder, commands::submit_transfer, constants::ETH, fee_estimator::FeeMode,
        provider::checked_http_provider, transfer_builder::TransferPayload,
    },
    session::{Session, SessionAccess, SessionKeeper, SessionRevocation},
    wallet::Wallet,
};

const TOKEN: Address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
const SPENDER: Address = address!("2000000000000000000000000000000000000000");

fn grant(topic: &str, address: Address, methods: &[&str]) -> WalletConnectGrant {
    serde_json::from_value(json!({
        "topic": topic,
        "address": address,
        "chains": [{"id": 1, "methods": methods}],
    }))
    .unwrap()
}

#[test]
fn rejects_sender_chain_and_calldata_mismatches() {
    let mut tx = json!({"from": SPENDER, "to": TOKEN, "chainId": "0x1", "data": "0xab"});
    let parsed = parse_transaction(&tx, SPENDER, 1).unwrap();
    assert_eq!(parsed.input.input().unwrap().as_ref(), &[0xab]);
    assert!(parse_transaction(&tx, TOKEN, 1).is_err());
    assert!(parse_transaction(&tx, SPENDER, 8453).is_err());

    tx["input"] = json!("0xcd");
    assert!(parse_transaction(&tx, SPENDER, 1).is_err());

    tx.as_object_mut().unwrap().remove("input");
    tx["authorizationList"] = json!([]);
    assert!(parse_transaction(&tx, SPENDER, 1).is_err());
}

#[test]
fn testnet_and_tor_settings_are_enforced() {
    let mut config: Config = serde_json::from_value(json!({"eth": {"anvil": false}})).unwrap();
    let settings = WalletConnectConfig::enabled(&config).unwrap();
    assert!(settings.rpc_url(8453).is_ok());
    assert!(settings.rpc_url(84532).is_err());

    config.eth.walletconnect_testnets = true;
    let settings = WalletConnectConfig::from(&config);
    assert!(settings.rpc_url(1).is_err());
    assert!(settings.rpc_url(84532).is_ok());

    config.tor.enabled = true;
    assert!(WalletConnectConfig::enabled(&config).is_err());
}

#[test]
fn config_changes_reset_walletconnect_without_notifying_when_tor_is_enabled() {
    let current = Config::default();
    let settings = WalletConnectConfig::from(&current);
    let mut unrelated = current.clone();
    unrelated.tor.socks5_proxy = "socks5://127.0.0.1:9051".into();
    unrelated.security.session_inactivity_timeout_mins += 1;
    assert_eq!(
        settings.change_to(&WalletConnectConfig::from(&unrelated)),
        WalletConnectConfigChange::Unchanged
    );

    for ethereum in [
        json!({"rpc_url": "https://rpc.example"}),
        json!({"anvil": true}),
        json!({"walletconnect_project_id": "other-project"}),
        json!({"walletconnect_testnets": true}),
    ] {
        let mut next = current.clone();
        next.eth = serde_json::from_value(ethereum).unwrap();
        assert_eq!(
            settings.change_to(&WalletConnectConfig::from(&next)),
            WalletConnectConfigChange::ResetAndNotify
        );
    }

    let mut tor = current.clone();
    tor.tor.enabled = true;
    let tor_settings = WalletConnectConfig::from(&tor);
    assert_eq!(
        settings.change_to(&tor_settings),
        WalletConnectConfigChange::ResetOffline
    );
    assert_eq!(
        tor_settings.change_to(&settings),
        WalletConnectConfigChange::ResetAndNotify
    );
}

#[test]
fn walletconnect_grant_replacement_discards_stale_requests() {
    let config = Config::default();
    let settings = WalletConnectConfig::from(&config);
    let access = SessionAccess::default();
    let topic = "1".repeat(64);
    let request: WalletConnectRequest = serde_json::from_value(json!({
        "topic": topic,
        "id": "2",
        "chain": 1,
        "method": "personal_sign",
        "params_json": "[]",
        "expires_at": 120,
    }))
    .unwrap();
    let prepared = || PreparedRequest {
        action: Action::Message(Bytes::from_static(b"Review this message")),
        review: WalletConnectReview {
            title: "Sign message".into(),
            ..Default::default()
        },
    };
    let mut state = WalletConnectState::default();
    state
        .authorize(grant(&topic, SPENDER, &["personal_sign"]), SPENDER)
        .unwrap();
    let original = state.begin_request(&request, SPENDER, 1).unwrap();
    let claim = |state: &mut WalletConnectState| {
        state.claim_approval(&topic, &request.id, &settings, access.clone(), 1)
    };
    assert_eq!(
        claim(&mut state).err().as_deref(),
        Some("The WalletConnect request is still being prepared")
    );

    state.disconnect(&topic);
    assert!(original.cancelled.is_cancelled());
    state
        .authorize(grant(&topic, SPENDER, &["personal_sign"]), SPENDER)
        .unwrap();
    let replacement = state.begin_request(&request, SPENDER, 1).unwrap();
    assert!(state.finish_request(&original, Ok(prepared())).is_err());
    state.finish_request(&replacement, Ok(prepared())).unwrap();
    let approval = claim(&mut state).unwrap();
    assert!(claim(&mut state).is_err());
    let signed = state.take_confirmation(&approval).unwrap();
    assert!(!signed.is_cancelled());
    assert!(state.take_confirmation(&approval).is_none());
    assert!(state.begin_request(&request, SPENDER, 1).is_err());
    state.disconnect(&topic);
    assert!(signed.is_cancelled());
}

#[test]
fn walletconnect_expired_approval_cannot_consume_reused_request_id() {
    let config = Config::default();
    let settings = WalletConnectConfig::from(&config);
    let access = SessionAccess::default();
    let topic = "1".repeat(64);
    let request = |expires_at| {
        serde_json::from_value(json!({
            "topic": topic,
            "id": "2",
            "chain": 1,
            "method": "personal_sign",
            "params_json": "[]",
            "expires_at": expires_at,
        }))
        .unwrap()
    };
    let mut state = WalletConnectState::default();
    state
        .authorize(grant(&topic, SPENDER, &["personal_sign"]), SPENDER)
        .unwrap();
    let mut prepare_and_claim = |expires_at, now| {
        let preparation = state
            .begin_request(&request(expires_at), SPENDER, now)
            .unwrap();
        let prepared = PreparedRequest {
            action: Action::Message(Bytes::from_static(b"Review this message")),
            review: WalletConnectReview::default(),
        };
        state.finish_request(&preparation, Ok(prepared)).unwrap();
        state
            .claim_approval(&topic, "2", &settings, access.clone(), now)
            .unwrap()
    };

    let expired = prepare_and_claim(120, 1);
    let replacement = prepare_and_claim(240, 120);
    assert!(expired.cancelled.is_cancelled());
    assert!(state.take_confirmation(&expired).is_none());
    assert!(!replacement.cancelled.is_cancelled());
    state.reject(&topic, "2");
    assert!(replacement.cancelled.is_cancelled());
    assert!(state.take_confirmation(&replacement).is_none());
}

#[tokio::test]
async fn approves_exact_contract_call_once_on_anvil() {
    let mut config = Config::default();
    let anvil = Anvil::new()
        .path(config.eth.anvil_bin())
        .chain_id(1)
        .spawn();
    config.eth.rpc_url = anvil.endpoint();
    config.eth.walletconnect_project_id = "local-test".into();

    let provider = checked_http_provider(&config.eth.rpc_url, 1).await.unwrap();
    assert!(
        checked_http_provider(&config.eth.rpc_url, 8453)
            .await
            .is_err()
    );
    let wallet = Wallet::new(
        config.clone(),
        "test-walletconnect".into(),
        "test test test test test test test test test test test junk".into(),
        "test-walletconnect".into(),
        None,
    )
    .unwrap();
    let mut session = Session::new(wallet, Duration::from_secs(120));
    let address = active_address(&session).unwrap();

    provider
        .anvil_set_balance(address, U256::from(1_000_000_000_000_000_000u64))
        .await
        .unwrap();
    provider.anvil_mine(Some(1), None).await.unwrap();
    let mut transfer = TxBuilder::new(provider.clone());
    let transfer_payload = TransferPayload {
        token: ETH.clone(),
        raw_amount: "0.001".into(),
        sender: address,
        recipient: SPENDER,
        fee_mode: FeeMode::Standard,
    };
    transfer
        .create_transfer(transfer_payload.clone())
        .await
        .unwrap();
    // Persist the approve() amount from calldata to storage slot 0.
    provider
        .anvil_set_code(
            TOKEN,
            Bytes::from_static(&[0x60, 0x24, 0x35, 0x60, 0x00, 0x55, 0x00]),
        )
        .await
        .unwrap();
    let allowance = U256::from(250);
    let mut calldata = vec![0x09, 0x5e, 0xa7, 0xb3];
    calldata.extend([0; 12]);
    calldata.extend(SPENDER.as_slice());
    calldata.extend(allowance.to_be_bytes::<32>());
    let topic = "1".repeat(64);
    let request: WalletConnectRequest = serde_json::from_value(json!({
        "topic": topic,
        "id": "2",
        "chain": 1,
        "method": "eth_sendTransaction",
        "params_json": json!([{
            "from": address, "to": TOKEN, "data": Bytes::from(calldata),
        }]).to_string(),
        "expires_at": now() + 120,
    }))
    .unwrap();
    let grant = grant(&topic, address, &["eth_sendTransaction", "personal_sign"]);
    grant.validate(&WalletConnectConfig::from(&config)).unwrap();
    session.walletconnect.authorize(grant, address).unwrap();
    let revocation = SessionRevocation::default();
    let sk = SessionKeeper::with_revocation(None, None, revocation.clone());
    let access = {
        let mut keeper = sk.lock().await;
        keeper.set(session);
        keeper.unexpired_session().unwrap().access().clone()
    };
    let config = Mutex::new(config);

    let review = prepare_request(&request, &config, &sk).await.unwrap();
    assert_eq!(review.spender, Some(SPENDER.to_string()));
    assert!(
        review
            .allowance
            .as_deref()
            .unwrap()
            .starts_with("0.000250 USDC")
    );
    assert!(review.fee_eth.is_some());
    async fn unexpected_confirmation(_: &Approval) -> Result<(), String> {
        panic!("unexpected confirmation prompt");
    }
    assert!(
        approve_request(&topic, "3", &config, &sk, unexpected_confirmation)
            .await
            .is_err()
    );
    let hash = approve_request(&topic, &request.id, &config, &sk, async |approval| {
        assert!(
            approve_request(&topic, &request.id, &config, &sk, unexpected_confirmation)
                .await
                .is_err()
        );
        let description = approval
            .prepared
            .review
            .description(approval.address, approval.chain);
        assert!(description.contains("Allowance: 0.000250 USDC"));
        approval.ensure_active()
    })
    .await
    .unwrap()
    .unwrap();
    assert!(
        approve_request(&topic, &request.id, &config, &sk, unexpected_confirmation)
            .await
            .is_err()
    );
    provider.anvil_mine(Some(1), None).await.unwrap();
    let receipt = provider
        .get_transaction_receipt(hash.parse().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert!(receipt.status());
    assert_eq!(
        provider.get_storage_at(TOKEN, U256::ZERO).await.unwrap(),
        allowance
    );
    assert_eq!(provider.get_transaction_count(address).await.unwrap(), 1);

    assert_eq!(
        submit_transfer(&mut transfer, address, &sk, &access)
            .await
            .unwrap_err(),
        "The active account or nonce changed. Estimate the transfer again."
    );
    transfer.create_transfer(transfer_payload).await.unwrap();
    let (hash, (wallet_name, account)) = submit_transfer(&mut transfer, address, &sk, &access)
        .await
        .unwrap();
    assert_eq!(wallet_name, "test-walletconnect");
    assert_eq!(account, 0);
    provider.anvil_mine(Some(1), None).await.unwrap();
    assert!(
        provider
            .get_transaction_receipt(hash)
            .await
            .unwrap()
            .unwrap()
            .status()
    );
    assert_eq!(
        provider.get_balance(SPENDER).await.unwrap(),
        U256::from(1_000_000_000_000_000u64),
    );

    let message_bytes = Bytes::from_static(b"WalletConnect approval test");
    let message = |id| {
        serde_json::from_value(json!({
            "topic": topic,
            "id": id,
            "chain": 1,
            "method": "personal_sign",
            "params_json": json!([message_bytes, address]).to_string(),
            "expires_at": now() + 120,
        }))
        .unwrap()
    };
    prepare_request(&message("3"), &config, &sk).await.unwrap();
    let rejected = approve_request(&topic, "3", &config, &sk, async |_| {
        Err("User rejected the wallet action".into())
    })
    .await;
    assert_eq!(rejected.unwrap_err(), "User rejected the wallet action");

    prepare_request(&message("4"), &config, &sk).await.unwrap();
    let result = approve_request(&topic, "4", &config, &sk, async |approval| {
        approval.ensure_active()
    })
    .await
    .unwrap()
    .unwrap();
    let signature: Signature = result.parse().unwrap();
    assert_eq!(
        signature.recover_address_from_msg(&message_bytes).unwrap(),
        address
    );

    prepare_request(&message("5"), &config, &sk).await.unwrap();
    let interrupted = approve_request(&topic, "5", &config, &sk, async |approval| {
        revocation.revoke();
        assert!(approval.ensure_active().is_err());
        Ok(())
    })
    .await;
    assert!(interrupted.is_err());
}
