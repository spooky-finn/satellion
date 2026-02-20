use diesel::{SqliteConnection, prelude::*, r2d2::ConnectionManager, result::Error};
use r2d2::Pool;

use crate::{db::BlockHeader, repository::BaseRepository, schema::bitcoin_block_headers};

#[derive(Clone)]
pub struct ChainRepository {
    base: BaseRepository,
}

impl ChainRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            base: BaseRepository::new(db_pool),
        }
    }

    // pub fn save_block_header(&self, block_header: &IndexedHeader) -> Result<usize, Error> {
    //     let mut conn = self.base.get_conn()?;
    //     diesel::insert_into(bitcoin_block_headers::table)
    //         .values(&BlockHeader {
    //             height: block_header.height as i32,
    //             merkle_root: block_header.header.merkle_root.to_string(),
    //             prev_blockhash: block_header.header.prev_blockhash.to_string(),
    //             time: block_header.header.time as i32,
    //             version: block_header.header.version.to_consensus(),
    //             bits: block_header.header.bits.to_consensus() as i32,
    //             nonce: block_header.header.nonce as i32,
    //         })
    //         .execute(&mut conn)
    //         .or_else(|err| {
    //             if err.to_string().contains("UNIQUE constraint failed") {
    //                 Ok(0)
    //             } else {
    //                 Err(err)
    //             }
    //         })
    // }

    pub fn last_block(&self) -> Result<BlockHeader, Error> {
        let mut conn = self.base.get_conn()?;
        let last_block = bitcoin_block_headers::table
            .select(bitcoin_block_headers::all_columns)
            .order(bitcoin_block_headers::height.desc())
            .first::<BlockHeader>(&mut conn)?;

        Ok(last_block)
    }

    // pub fn get_block_headers(
    //     &self,
    //     last_seen_height: u32,
    //     limit: i64,
    // ) -> Result<Vec<BlockHeader>, Error> {
    //     let mut conn = self.base.get_conn()?;
    //     let min_height = (last_seen_height as i64 - limit) as i32;
    //     bitcoin_block_headers::table
    //         .select(bitcoin_block_headers::all_columns)
    //         .filter(bitcoin_block_headers::height.gt(min_height))
    //         .filter(bitcoin_block_headers::height.le(last_seen_height as i32))
    //         .order(bitcoin_block_headers::height.desc())
    //         .load::<BlockHeader>(&mut conn)
    // }

    pub fn get_block_header(&self, height: u32) -> Result<Option<BlockHeader>, Error> {
        let mut conn = self.base.get_conn()?;
        bitcoin_block_headers::table
            .select(bitcoin_block_headers::all_columns)
            .filter(bitcoin_block_headers::height.eq(height as i32))
            .first::<BlockHeader>(&mut conn)
            .optional()
    }
}
