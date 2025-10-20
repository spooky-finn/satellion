use crate::db;
use crate::db::BlockHeader;
use crate::schema;
use bip157::chain::IndexedHeader;
use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use r2d2::PooledConnection;

pub struct Repository {
    pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Repository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { db_pool }
    }

    fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<diesel::SqliteConnection>>, diesel::result::Error>
    {
        self.db_pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })
    }

    pub fn save_block_header(
        &self,
        block_header: IndexedHeader,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        diesel::insert_into(schema::bitcoin_block_headers::table)
            .values(&BlockHeader {
                height: block_header.height as i32,
                merkle_root: block_header.header.merkle_root.to_string(),
                prev_blockhash: block_header.header.prev_blockhash.to_string(),
                time: block_header.header.time as i32,
                version: block_header.header.version.to_consensus(),
                bits: block_header.header.bits.to_consensus() as i32,
                nonce: block_header.header.nonce as i32,
            })
            .execute(&mut conn)
    }

    pub fn load_block_headers(
        &self,
        limit: i64,
    ) -> Result<Vec<BlockHeader>, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        schema::bitcoin_block_headers::table
            .select(schema::bitcoin_block_headers::all_columns)
            .limit(limit)
            .order(schema::bitcoin_block_headers::height.desc())
            .load::<BlockHeader>(&mut conn)
    }

    pub fn wallet_exist(&self) -> Result<bool, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        schema::keys::table
            .select(diesel::dsl::count(schema::keys::id))
            .first::<i64>(&mut conn)
            .map_err(|_| diesel::result::Error::NotFound)
            .map(|count| count > 0)
    }

    pub fn save_private_key(
        &self,
        key: String,
        name: String,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let key = db::Key {
            id: None,
            name: Some(name),
            prk: key,
            created_at: chrono::Utc::now().to_string(),
        };
        diesel::insert_into(schema::keys::table)
            .values(&key)
            .execute(&mut conn)
    }
}
