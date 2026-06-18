use specta_typescript::Typescript;

use crate::{
    chain::{btc, eth},
    commands, config, event_emitter,
};

pub fn handlers() -> tauri_specta::Builder {
    #[cfg(debug_assertions)]
    {
        use crate::chain::{btc, eth};

        let lang = Typescript::default().formatter(specta_typescript::formatter::biome);
        tauri_specta::Builder::<tauri::Wry>::new()
            .commands(tauri_specta::collect_commands![
                commands::generate_mnemonic,
                commands::mnemonic_wordlist,
                commands::create_wallet,
                commands::get_wallets,
                commands::unlock_wallet,
                commands::unlock_wallet_with_biometric,
                commands::is_biometric_unlock_supported,
                commands::is_biometric_unlock_enabled,
                commands::enable_biometric_unlock,
                commands::disable_biometric_unlock,
                commands::rename_wallet,
                commands::forget_wallet,
                commands::get_config,
                commands::get_config_schema,
                commands::set_config,
                commands::add_account,
                commands::switch_account,
                commands::rename_account,
                commands::switch_blockchain,
                commands::price_feed,
                commands::validate_address,
                commands::list_transactions,
            ])
            .constant("MIN_PASSPHRASE_LEN", config::MIN_PASSPHRASE_LEN)
            .events(event_emitter::list_events())
            .export(lang.clone(), "../src/bindings/index.ts")
            .expect("Failed to export TypeScript bindings");

        tauri_specta::Builder::<tauri::Wry>::new()
            .commands(tauri_specta::collect_commands![
                btc::commands::derive_external_address,
                btc::commands::next_unused_index,
                btc::commands::get_external_addresess,
                btc::commands::get_utxos,
                btc::commands::sync_utxos,
                btc::commands::discover_wallet,
                btc::commands::account_info,
                btc::commands::build_tx,
                btc::commands::broadcast_tx,
                btc::commands::bump_fee_cpfp,
            ])
            .export(lang.clone(), "../src/bindings/btc.ts")
            .expect("Failed to export TypeScript bindings");

        tauri_specta::Builder::<tauri::Wry>::new()
            .commands(tauri_specta::collect_commands![
                eth::commands::get_network_status,
                eth::commands::get_wallet_balance,
                eth::commands::estimate_transfer,
                eth::commands::execute_transfer,
                eth::commands::track_token,
                eth::commands::untrack_token,
                eth::commands::anvil_set_initial_balances,
            ])
            .export(lang, "../src/bindings/eth.ts")
            .expect("Failed to export TypeScript bindings");
    }

    // Merged builder for runtime - chains .commands() calls
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::generate_mnemonic,
        commands::mnemonic_wordlist,
        commands::create_wallet,
        commands::get_wallets,
        commands::unlock_wallet,
        commands::unlock_wallet_with_biometric,
        commands::is_biometric_unlock_supported,
        commands::is_biometric_unlock_enabled,
        commands::enable_biometric_unlock,
        commands::disable_biometric_unlock,
        commands::rename_wallet,
        commands::forget_wallet,
        commands::get_config,
        commands::get_config_schema,
        commands::set_config,
        commands::add_account,
        commands::switch_account,
        commands::rename_account,
        commands::switch_blockchain,
        commands::price_feed,
        commands::validate_address,
        commands::list_transactions,
        //
        btc::commands::derive_external_address,
        btc::commands::next_unused_index,
        btc::commands::get_external_addresess,
        btc::commands::get_utxos,
        btc::commands::sync_utxos,
        btc::commands::discover_wallet,
        btc::commands::account_info,
        btc::commands::build_tx,
        btc::commands::broadcast_tx,
        btc::commands::bump_fee_cpfp,
        //
        eth::commands::get_network_status,
        eth::commands::get_wallet_balance,
        eth::commands::estimate_transfer,
        eth::commands::execute_transfer,
        eth::commands::track_token,
        eth::commands::untrack_token,
        eth::commands::anvil_set_initial_balances,
    ])
}
