// @generated automatically by Diesel CLI.

diesel::table! {
    #[sql_name = "bitcoin.block_headers"]
    bitcoin_block_headers (height) {
        height -> Integer,
        blockhash -> Text,
        prev_blockhash -> Text,
        time -> Integer,
    }
}
