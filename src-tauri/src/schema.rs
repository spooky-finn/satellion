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
