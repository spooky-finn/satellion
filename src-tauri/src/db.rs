use crate::config::Config;
use crate::schema;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

// Nice mapping of Diesel to Rust types:
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
pub type Pool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

// Embed migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Ensure the data directory exists
pub fn ensure_data_dir() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = Config::config_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    Ok(())
}

pub fn initialize_database() {
    ensure_data_dir().expect("cannot create data dir");

    let db_path = Config::db_path().expect("faile to get db path");

    // Create database file if it doesn't exist
    if !db_path.exists() {
        std::fs::File::create(&db_path).expect("failed to create database file");
    }

    // Run migrations
    let mut conn = SqliteConnection::establish(&db_path.to_string_lossy())
        .expect("cannot create db connection");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

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

#[derive(Insertable, Queryable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = schema::tokens)]
pub struct Token {
    pub wallet_id: i32,
    pub chain: i32,
    pub symbol: String,
    pub address: Vec<u8>,
    pub decimals: i32,
}
