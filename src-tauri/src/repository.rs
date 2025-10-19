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
        diesel::insert_into(schema::block_headers::table)
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
        schema::block_headers::table
            .select(schema::block_headers::all_columns)
            .limit(limit)
            .order(schema::block_headers::height.desc())
            .load::<BlockHeader>(&mut conn)
    }
}
