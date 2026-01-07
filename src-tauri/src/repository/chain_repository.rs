use std::time::{SystemTime, UNIX_EPOCH};

use bip157::chain::IndexedHeader;
use diesel::{SqliteConnection, prelude::*, r2d2::ConnectionManager, result::Error};
use r2d2::Pool;

use crate::{
    btc::utxo::UTxO,
    db::{self, BlockHeader},
    repository::BaseRepository,
    schema,
};

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

    pub fn save_block_header(&self, block_header: IndexedHeader) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
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

    pub fn get_block_headers(&self, limit: i64) -> Result<Vec<BlockHeader>, Error> {
        let mut conn = self.base.get_conn()?;
        schema::bitcoin_block_headers::table
            .select(schema::bitcoin_block_headers::all_columns)
            .limit(limit)
            .order(schema::bitcoin_block_headers::height.desc())
            .load::<BlockHeader>(&mut conn)
    }

    pub async fn insert_utxos(&self, txs: Vec<UTxO>) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        let utxos: Vec<db::Utxo> = txs
            .iter()
            .map(|t| db::Utxo {
                txid: t.tx_hash.to_string(),
                script_pubkey: t.output.script_pubkey.to_string(),
                deriviation_path: t.derive_path.to_string(),
                value: t.output.value.to_sat() as i64,
                vout: t.vout_idx as i32,
                block_hash: t.block_hash.to_string(),
                block_height: t.block_height as i32,
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64,
                spent: 0,
                spent_at: None,
            })
            .collect();
        diesel::insert_into(schema::utxos::table)
            .values(&utxos)
            .execute(&mut conn)
    }
}
