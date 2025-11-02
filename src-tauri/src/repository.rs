use crate::db;
use crate::db::BlockHeader;
use crate::schema;
use bip157::chain::IndexedHeader;
use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use r2d2::PooledConnection;

#[derive(Clone)]
pub struct Repository {
    pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

type Error = diesel::result::Error;

#[derive(serde::Serialize)]
pub struct AvailableWallet {
    pub id: i32,
    pub name: Option<String>,
}

impl Repository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { db_pool }
    }

    fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<diesel::SqliteConnection>>, Error> {
        self.db_pool.get().map_err(|e| {
            Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })
    }

    pub fn available_wallets(&self) -> Result<Vec<AvailableWallet>, Error> {
        let mut conn = self.get_conn()?;
        let wallets = schema::wallets::table
            .select((schema::wallets::id, schema::wallets::name))
            .load::<(i32, Option<String>)>(&mut conn)?;

        let result = wallets
            .into_iter()
            .map(|(id, name)| AvailableWallet { id, name })
            .collect();
        Ok(result)
    }

    pub fn save_block_header(&self, block_header: IndexedHeader) -> Result<usize, Error> {
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

    pub fn load_block_headers(&self, limit: i64) -> Result<Vec<BlockHeader>, Error> {
        let mut conn = self.get_conn()?;
        schema::bitcoin_block_headers::table
            .select(schema::bitcoin_block_headers::all_columns)
            .limit(limit)
            .order(schema::bitcoin_block_headers::height.desc())
            .load::<BlockHeader>(&mut conn)
    }

    pub fn insert_wallet(&self, wallet: db::Wallet) -> Result<usize, Error> {
        let mut conn = self.get_conn()?;
        diesel::insert_into(schema::wallets::table)
            .values(&wallet)
            .execute(&mut conn)
    }

    pub fn get_wallet_by_id(&self, wallet_id: i32) -> Result<db::Wallet, Error> {
        let mut conn = self.get_conn()?;
        let wallet = schema::wallets::table
            .filter(schema::wallets::id.eq(wallet_id))
            .select(schema::wallets::all_columns)
            .first::<db::Wallet>(&mut conn)?;
        Ok(wallet)
    }

    pub fn delete_wallet(&self, wallet_id: i32) -> Result<usize, Error> {
        let mut conn = self.get_conn()?;
        diesel::delete(schema::wallets::table.filter(schema::wallets::id.eq(wallet_id)))
            .execute(&mut conn)
    }

    pub fn last_wallet_id(&self) -> Result<i32, Error> {
        let mut conn = self.get_conn()?;
        let wallet_id = schema::wallets::table
            .select(schema::wallets::id)
            .order(schema::wallets::id.desc())
            .first::<i32>(&mut conn);

        match wallet_id {
            Ok(id) => Ok(id),
            Err(e) => {
                if e.to_string().contains("Record not found") {
                    Ok(0)
                } else {
                    Err(e)
                }
            }
        }
    }
}
