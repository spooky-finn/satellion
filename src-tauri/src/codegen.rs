use specta_typescript::Typescript;

use crate::{btc, commands, config, eth, event_emitter};

pub fn handlers() -> tauri_specta::Builder {
    let lang = Typescript::default().formatter(specta_typescript::formatter::biome);

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
            btc::commands::derive_external_address,
            btc::commands::unoccupied_deriviation_index,
            btc::commands::get_external_addresess,
            btc::commands::get_utxos,
            btc::commands::sync_utxos,
            btc::commands::account_info,
            btc::commands::build_tx,
            btc::commands::send_tx,
        ])
        .export(lang.clone(), "../src/bindings/btc.ts")
        .expect("Failed to export TypeScript bindings");

    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            eth::commands::get_network_status,
            eth::commands::get_wallet_balance,
            eth::commands::estimate_transfer,
            eth::commands::execute_transfer,
            eth::commands::verify_address,
            eth::commands::track_token,
            eth::commands::untrack_token,
            eth::commands::anvil_set_initial_balances,
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
        btc::commands::derive_external_address,
        btc::commands::unoccupied_deriviation_index,
        btc::commands::get_external_addresess,
        btc::commands::get_utxos,
        btc::commands::sync_utxos,
        btc::commands::account_info,
        btc::commands::build_tx,
        btc::commands::send_tx,
        //
        eth::commands::get_network_status,
        eth::commands::get_wallet_balance,
        eth::commands::estimate_transfer,
        eth::commands::execute_transfer,
        eth::commands::verify_address,
        eth::commands::track_token,
        eth::commands::untrack_token,
        eth::commands::anvil_set_initial_balances,
    ])
}
