use crate::config::Chain;
use crate::db;
use crate::repository::BaseRepository;
use crate::schema;
use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error;
use r2d2::Pool;

#[derive(Clone)]
pub struct TokenRepository {
    base: BaseRepository,
}

impl TokenRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            base: BaseRepository::new(db_pool),
        }
    }

    /// Load all tokens for a given wallet and chain
    pub fn load(&self, wallet_id: i32, chain: Chain) -> Result<Vec<db::Token>, Error> {
        let mut conn = self.base.get_conn()?;
        schema::tokens::table
            .filter(schema::tokens::wallet_id.eq(wallet_id))
            .filter(schema::tokens::chain.eq(i32::from(chain)))
            .select(schema::tokens::all_columns)
            .load::<db::Token>(&mut conn)
    }

    /// Insert a single token
    pub fn insert(&self, token: db::Token) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        diesel::insert_into(schema::tokens::table)
            .values(&token)
            .execute(&mut conn)
    }

    /// Insert many tokens, ignoring duplicates (based on primary key: wallet_id, chain, address)
    pub fn insert_or_ignore_many(&self, tokens: Vec<db::Token>) -> Result<usize, Error> {
        if tokens.is_empty() {
            return Ok(0);
        }
        let mut conn = self.base.get_conn()?;
        conn.transaction(|conn| {
            let mut inserted = 0;
            for token in &tokens {
                let result = diesel::insert_into(schema::tokens::table)
                    .values(token)
                    .on_conflict_do_nothing()
                    .execute(conn)?;
                inserted += result;
            }
            Ok(inserted)
        })
    }

    pub fn remove(&self, wallet_id: i32, chain: Chain, address: &[u8]) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        diesel::delete(schema::tokens::table)
            .filter(schema::tokens::wallet_id.eq(wallet_id))
            .filter(schema::tokens::chain.eq(i32::from(chain)))
            .filter(schema::tokens::address.eq(address))
            .execute(&mut conn)
    }
}
