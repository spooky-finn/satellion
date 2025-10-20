use crate::schema;
use diesel::prelude::*;

// Nice mapping of Diesel to Rust types:
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
pub type Pool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[diesel(table_name = schema::keys)]
pub struct Key {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub prk: String,
    pub created_at: String,
}

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[diesel(table_name = schema::bitcoin_block_headers)]
pub struct BlockHeader {
    pub height: i32,
    pub merkle_root: String,
    pub prev_blockhash: String,
    pub time: i32,
    pub version: i32,
    pub bits: i32,
    pub nonce: i32,
}
