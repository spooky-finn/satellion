use diesel::{prelude::*, r2d2::ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::{config::Config, schema};

// Nice mapping of Diesel to Rust types:
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
pub type Pool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>;

// Embed migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Ensure the data directory exists
pub fn ensure_data_dir() -> std::io::Result<()> {
    let config_dir = Config::config_dir();
    std::fs::create_dir_all(&config_dir)?;
    Ok(())
}

pub fn initialize() {
    ensure_data_dir().expect("cannot create data dir");
    let db_path = Config::db_path();

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
    let manager =
        ConnectionManager::<SqliteConnection>::new(Config::db_path().to_string_lossy().to_string());
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Error creating DB pool")
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

#[derive(Insertable, Queryable, Debug, PartialEq, Clone)]
#[diesel(table_name = schema::utxos)]
pub struct Utxo {
    /// Transaction hash (32 bytes)
    pub txid: String,
    /// Output index within the transaction
    pub vout: i32,
    /// Value in satoshis
    pub value: i64,
    /// ScriptPubKey (raw hex)
    pub script_pubkey: String,
    /// Block height where this UTXO was created
    pub block_height: i32,
    /// Block hash for additional integrity
    pub block_hash: String,
    /// Whether the output has been spent (0 = unspent, 1 = spent)
    pub spent: i32,
    /// Timestamp when UTXO was created (unix seconds)
    pub created_at: i64,
    /// Timestamp when UTXO was spent (unix seconds, null if unspent)
    pub spent_at: Option<i64>,
}
