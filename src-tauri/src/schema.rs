// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (tx_hash) {
        tx_hash -> Text,
        wallet_name -> Text,
        chain -> Text,
        account_index -> Integer,
        direction -> SmallInt,
        status -> Text,
        from_address -> Nullable<Text>,
        to_address -> Nullable<Text>,
        amount -> BigInt,
        fee -> Nullable<Integer>,
        block_height -> Nullable<BigInt>,
        chain_data -> Text,
        created_at -> BigInt,
        confirmed_at -> Nullable<BigInt>,
    }
}

diesel::table! {
    btc_broadcast_inputs (network, outpoint) {
        network -> Text,
        outpoint -> Text,
        txid -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    btc_broadcast_payloads (wallet_name, account_index, network, txid) {
        txid -> Text,
        wallet_name -> Text,
        account_index -> Integer,
        network -> Text,
        raw_transaction -> Text,
        wallet_key -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(transactions, btc_broadcast_payloads);
