// @generated automatically by Diesel CLI.

diesel::table! {
    block_headers (height) {
        height -> Integer,
        merkle_root -> Text,
        prev_blockhash -> Text,
        time -> Integer,
        version -> Integer,
        bits -> Integer,
        nonce -> Integer,
    }
}
