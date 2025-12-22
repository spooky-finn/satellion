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
    utxos (txid, vout) {
        txid -> Text,
        vout -> Integer,
        value -> BigInt,
        script_pubkey -> Text,
        block_height -> Integer,
        block_hash -> Text,
        spent -> Integer,
        created_at -> BigInt,
        spent_at -> Nullable<BigInt>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(bitcoin_block_headers, utxos,);
