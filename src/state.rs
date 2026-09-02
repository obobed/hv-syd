use sqlx::{SqlitePool, pool::PoolOptions};

use crate::Config;

const MAX_DATABASE_CONNECTIONS: u32 = 32;
const MIN_DATABASE_CONNECTIONS: u32 = 4;
const DATABASE_URL: &str = "sqlite:todos.db";

pub(crate) struct State {
    pub config: Config,

    pub database_pool: SqlitePool,
}

impl State {
    pub(crate) fn new(config: Config) -> Result<State, sqlx::Error> {
        let database_pool = PoolOptions::new()
            .max_connections(MAX_DATABASE_CONNECTIONS)
            .min_connections(MIN_DATABASE_CONNECTIONS)
            .connect_lazy(DATABASE_URL)?;

        Ok(State {
            config,
            database_pool,
        })
    }
}
