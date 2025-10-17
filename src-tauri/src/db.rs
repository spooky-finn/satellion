use crate::schema;
use diesel::prelude::*;

// Nice mapping of Diesel to Rust types:
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[diesel(table_name = schema::blocks)]
pub struct Block {
    pub height: i32,
    pub merkle_root: String,
    pub prev_blockhash: String,
    pub time: i64,
}
