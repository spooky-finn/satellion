// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (wallet_name, chain, tx_hash) {
        tx_hash -> Text,
        wallet_name -> Text,
        chain -> Text,
        account_index -> Integer,
        direction -> SmallInt,
        status -> Text,
        from_address -> Nullable<Text>,
        to_address -> Nullable<Text>,
        amount -> Text,
        fee -> Nullable<Text>,
        block_height -> Nullable<BigInt>,
        chain_data -> Text,
        created_at -> BigInt,
        confirmed_at -> Nullable<BigInt>,
    }
}
