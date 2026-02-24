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

diesel::table! {
    #[sql_name = "bitcoin.compact_filters"]
    bitcoin_compact_filters (blockhash) {
        blockhash -> Text,
        filter_data -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(bitcoin_block_headers, bitcoin_compact_filters,);
