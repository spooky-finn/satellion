use bip157::chain::IndexedHeader;
use diesel::{SqliteConnection, prelude::*, r2d2::ConnectionManager, result::Error};
use r2d2::Pool;

use crate::{
    db::{BlockHeader, CompactFilter},
    repository::BaseRepository,
    schema::{bitcoin_block_headers, bitcoin_compact_filters},
};

#[derive(Clone)]
pub struct ChainRepository {
    base: BaseRepository,
}

pub trait ChainRepositoryTrait: Send + Sync {
    fn save_block_header(&self, block_header: &IndexedHeader) -> Result<usize, Error>;
    fn last_block(&self) -> Result<BlockHeader, Error>;
    fn get_block_header(&self, height: u32) -> Result<Option<BlockHeader>, Error>;
    fn save_compact_filter(&self, blockhash: &str, filter_data: &[u8]) -> Result<(), Error>;
    fn get_compact_filter(&self, blockhash: &str) -> Result<Option<CompactFilter>, Error>;
}

impl ChainRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            base: BaseRepository::new(db_pool),
        }
    }
}

impl ChainRepositoryTrait for ChainRepository {
    fn save_block_header(&self, block_header: &IndexedHeader) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        let blockhash = block_header.block_hash().to_string();

        diesel::insert_into(bitcoin_block_headers::table)
            .values(&BlockHeader {
                height: block_header.height as i32,
                blockhash,
                prev_blockhash: block_header.header.prev_blockhash.to_string(),
                time: block_header.header.time as i32,
            })
            .execute(&mut conn)
            .or_else(|err| {
                if err.to_string().contains("UNIQUE constraint failed") {
                    Ok(0)
                } else {
                    Err(err)
                }
            })
    }

    fn last_block(&self) -> Result<BlockHeader, Error> {
        let mut conn = self.base.get_conn()?;
        let last_block = bitcoin_block_headers::table
            .select(bitcoin_block_headers::all_columns)
            .order(bitcoin_block_headers::height.desc())
            .first::<BlockHeader>(&mut conn)?;

        Ok(last_block)
    }

    fn get_block_header(&self, height: u32) -> Result<Option<BlockHeader>, Error> {
        let mut conn = self.base.get_conn()?;
        bitcoin_block_headers::table
            .select(bitcoin_block_headers::all_columns)
            .filter(bitcoin_block_headers::height.eq(height as i32))
            .first::<BlockHeader>(&mut conn)
            .optional()
    }

    fn save_compact_filter(&self, blockhash: &str, filter_data: &[u8]) -> Result<(), Error> {
        let mut conn = self.base.get_conn()?;
        diesel::insert_into(bitcoin_compact_filters::table)
            .values(&CompactFilter {
                blockhash: blockhash.to_string(),
                filter_data: filter_data.to_vec(),
            })
            .execute(&mut conn)?;
        Ok(())
    }

    fn get_compact_filter(&self, blockhash: &str) -> Result<Option<CompactFilter>, Error> {
        let mut conn = self.base.get_conn()?;
        bitcoin_compact_filters::table
            .select(bitcoin_compact_filters::all_columns)
            .filter(bitcoin_compact_filters::blockhash.eq(blockhash))
            .first::<CompactFilter>(&mut conn)
            .optional()
    }
}
