use specta_typescript::Typescript;

use crate::{btc, commands, config, eth, event_emitter};

pub fn handlers() -> tauri_specta::Builder {
    let lang = Typescript::default().formatter(specta_typescript::formatter::prettier);

    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::get_wallets,
            commands::unlock_wallet,
            commands::forget_wallet,
            commands::get_config,
            commands::add_account,
            commands::switch_account,
            commands::switch_blockchain,
            commands::price_feed,
        ])
        .constant("MIN_PASSPHRASE_LEN", config::MIN_PASSPHRASE_LEN)
        .events(event_emitter::list_events())
        .export(lang.clone(), "../src/bindings/index.ts")
        .expect("Failed to export TypeScript bindings");

    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            btc::commands::btc_derive_external_address,
            btc::commands::btc_unoccupied_deriviation_index,
            btc::commands::btc_get_external_addresess,
            btc::commands::btc_get_utxos,
            btc::commands::btc_sync_utxos,
            btc::commands::btc_account_info,
            btc::commands::btc_build_tx,
            btc::commands::btc_send_tx,
        ])
        .export(lang.clone(), "../src/bindings/btc.ts")
        .expect("Failed to export TypeScript bindings");

    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            eth::commands::eth_chain_info,
            eth::commands::eth_get_balance,
            eth::commands::eth_build_transfer_tx,
            eth::commands::eth_sign_and_send_tx,
            eth::commands::eth_verify_address,
            eth::commands::eth_track_token,
            eth::commands::eth_untrack_token,
            eth::commands::eth_anvil_set_initial_balances,
        ])
        .export(lang, "../src/bindings/eth.ts")
        .expect("Failed to export TypeScript bindings");

    // Merged builder for runtime - chains .commands() calls
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::generate_mnemonic,
        commands::create_wallet,
        commands::get_wallets,
        commands::unlock_wallet,
        commands::forget_wallet,
        commands::get_config,
        commands::add_account,
        commands::switch_account,
        commands::switch_blockchain,
        commands::price_feed,
        //
        btc::commands::btc_derive_external_address,
        btc::commands::btc_unoccupied_deriviation_index,
        btc::commands::btc_get_external_addresess,
        btc::commands::btc_get_utxos,
        btc::commands::btc_sync_utxos,
        btc::commands::btc_account_info,
        btc::commands::btc_build_tx,
        btc::commands::btc_send_tx,
        //
        eth::commands::eth_chain_info,
        eth::commands::eth_get_balance,
        eth::commands::eth_build_transfer_tx,
        eth::commands::eth_sign_and_send_tx,
        eth::commands::eth_verify_address,
        eth::commands::eth_track_token,
        eth::commands::eth_untrack_token,
        eth::commands::eth_anvil_set_initial_balances,
    ])
}
