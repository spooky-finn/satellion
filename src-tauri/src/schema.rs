// @generated automatically by Diesel CLI.

diesel::table! {
    #[sql_name = "bitcoin.block_headers"]
    bitcoin_block_headers (height) {
        height -> Integer,
        merkle_root -> Text,
        prev_blockhash -> Text,
        time -> Integer,
        version -> Integer,
        bits -> Integer,
        nonce -> Integer,
    }
}

diesel::table! {
    wallets (id) {
        id -> Integer,
        name -> Nullable<Text>,
        encrypted_key -> Binary,
        key_wrapped -> Binary,
        kdf_salt -> Binary,
        version -> Integer,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(bitcoin_block_headers, wallets,);
