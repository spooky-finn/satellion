use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error;
use r2d2::Pool;
use r2d2::PooledConnection;

#[derive(Clone)]
pub struct BaseRepository {
    pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl BaseRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { db_pool }
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<diesel::SqliteConnection>>, Error> {
        self.db_pool.get().map_err(|e| {
            Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })
    }
}
