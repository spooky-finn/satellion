use crate::config::Chain;
use crate::db;
use crate::repository::BaseRepository;
use crate::schema;
use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error;
use r2d2::Pool;
use specta;

#[derive(Clone)]
pub struct WalletRepository {
    base: BaseRepository,
}

#[derive(serde::Serialize, specta::Type)]
pub struct AvailableWallet {
    pub id: i32,
    pub name: Option<String>,
}

impl WalletRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            base: BaseRepository::new(db_pool),
        }
    }

    pub fn list(&self) -> Result<Vec<AvailableWallet>, Error> {
        let mut conn = self.base.get_conn()?;
        let wallets = schema::wallets::table
            .select((schema::wallets::id, schema::wallets::name))
            .load::<(i32, Option<String>)>(&mut conn)?;

        let result = wallets
            .into_iter()
            .map(|(id, name)| AvailableWallet { id, name })
            .collect();
        Ok(result)
    }

    pub fn insert(&self, wallet: db::Wallet) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        diesel::insert_into(schema::wallets::table)
            .values(&wallet)
            .execute(&mut conn)
    }

    pub fn get(&self, wallet_id: i32) -> Result<db::Wallet, Error> {
        let mut conn = self.base.get_conn()?;
        let wallet = schema::wallets::table
            .filter(schema::wallets::id.eq(wallet_id))
            .select(schema::wallets::all_columns)
            .first::<db::Wallet>(&mut conn)?;
        Ok(wallet)
    }

    pub fn delete(&self, wallet_id: i32) -> Result<usize, Error> {
        let mut conn = self.base.get_conn()?;
        diesel::delete(schema::wallets::table.filter(schema::wallets::id.eq(wallet_id)))
            .execute(&mut conn)
    }

    pub fn set_last_used_chain(&self, wallet_id: i32, chain: Chain) -> Result<usize, String> {
        let mut conn = self
            .base
            .get_conn()
            .map_err(|e| format!("failed to aquire db connection {e}"))?;
        diesel::update(schema::wallets::table.filter(schema::wallets::id.eq(wallet_id)))
            .set(schema::wallets::last_used_chain.eq(i32::from(chain) as i16))
            .execute(&mut conn)
            .map_err(|e| format!("failed set_last_used_chain {e}"))
    }

    pub fn last_used_id(&self) -> Result<i32, Error> {
        let mut conn = self.base.get_conn()?;
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
