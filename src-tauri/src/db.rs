use crate::config::Config;
use crate::schema;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use specta::Type;

// Nice mapping of Diesel to Rust types:
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
pub type Pool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

pub fn connect() -> Pool {
    let manager = ConnectionManager::<SqliteConnection>::new(
        Config::db_path()
            .expect("Failed to get DB path")
            .to_string_lossy()
            .to_string(),
    );
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Error creating DB pool")
}

#[derive(Insertable, Queryable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = schema::wallets)]
pub struct Wallet {
    pub id: i32,
    pub name: Option<String>,
    pub encrypted_key: Vec<u8>,
    pub key_wrapped: Vec<u8>,
    pub kdf_salt: Vec<u8>,
    pub version: i32,
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

#[derive(Type, Serialize, Deserialize)]
pub enum Chain {
    Bitcoin = 0,
    Ethereum = 1,
}

impl From<i32> for Chain {
    fn from(value: i32) -> Self {
        match value {
            0 => Chain::Bitcoin,
            1 => Chain::Ethereum,
            _ => panic!("No default value for Chain. Invalid integer: {}", value),
        }
    }
}

impl From<Chain> for i32 {
    fn from(chain: Chain) -> Self {
        chain as i32
    }
}

#[derive(Insertable, Queryable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = schema::tokens)]
pub struct Token {
    pub wallet_id: i32,
    pub chain: i32,
    pub symbol: String,
    pub address: Vec<u8>,
    pub decimals: i32,
}
