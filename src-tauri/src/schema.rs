// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (height) {
        height -> Integer,
        merkle_root -> Text,
        prev_blockhash -> Text,
        time -> BigInt,
    }
}
