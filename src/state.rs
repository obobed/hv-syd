use heed::{
    EnvOpenOptions,
    types::{SerdeRmp, Str},
};
use sqlx::{SqlitePool, pool::PoolOptions};

use crate::{Config, Error, models::TokenMetadata};

const MAX_DATABASE_CONNECTIONS: u32 = 32;
const MIN_DATABASE_CONNECTIONS: u32 = 4;
const DATABASE_URL: &str = "sqlite:todos.db";
const TOKEN_STORE_PATH: &str = "tokens";

pub type TokenMap = heed::Database<Str, SerdeRmp<TokenMetadata>>;

pub(crate) struct State {
    pub config: Config,

    pub database_pool: SqlitePool,

    pub token_env: heed::Env,
    pub tokens: TokenMap,
}

impl State {
    pub(crate) fn new(config: Config) -> Result<State, Error> {
        let database_pool = PoolOptions::new()
            .max_connections(MAX_DATABASE_CONNECTIONS)
            .min_connections(MIN_DATABASE_CONNECTIONS)
            .connect_lazy(DATABASE_URL)?;

        // SAFETY:
        //     - The `NO_LOCK` flag is not set.
        //     - No remote filesystems,,, I don't know how I can enforce this :c
        //     - Transactions are finished as quickly as possible.
        let token_env = unsafe { EnvOpenOptions::new().open(TOKEN_STORE_PATH)? };
        let mut wtxn = token_env.write_txn()?;
        let tokens = token_env.create_database(&mut wtxn, None)?;
        wtxn.commit()?;

        Ok(State {
            config,
            database_pool,

            token_env,
            tokens,
        })
    }
}
