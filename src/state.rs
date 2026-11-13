use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use heed::{
    EnvOpenOptions,
    types::{SerdeRmp, Str},
};
use sqlx::{SqlitePool, pool::PoolOptions, sqlite::SqliteConnectOptions};

use crate::{Config, Error, models::TokenMetadata};

const MAX_DATABASE_CONNECTIONS: u32 = 32;
const MIN_DATABASE_CONNECTIONS: u32 = 4;
const DATABASE_PATH: &str = "database.db";
const TOKEN_STORE_PATH: &str = "tokens";

pub type TokenMap = heed::Database<Str, SerdeRmp<TokenMetadata>>;
pub type OtpMap = Arc<RwLock<HashMap<String, String>>>;

pub(crate) struct State {
    pub config: Config,

    pub database_pool: SqlitePool,

    pub token_env: heed::Env,
    pub tokens: TokenMap,

    pub otps: OtpMap,
}

impl State {
    pub(crate) fn new(config: Config) -> Result<State, Error> {
        let options = SqliteConnectOptions::default()
            .filename(DATABASE_PATH)
            .create_if_missing(true);

        let database_pool = PoolOptions::new()
            .max_connections(MAX_DATABASE_CONNECTIONS)
            .min_connections(MIN_DATABASE_CONNECTIONS)
            .connect_lazy_with(options);

        // SAFETY:
        //     - The `NO_LOCK` flag is not set.
        //     - No remote filesystems,,, I don't know how I can enforce this :c
        //     - Transactions are finished as quickly as possible.
        let token_env = unsafe { EnvOpenOptions::new().open(TOKEN_STORE_PATH)? };
        let mut wtxn = token_env.write_txn()?;
        let tokens = token_env.create_database(&mut wtxn, None)?;
        wtxn.commit()?;

        let otps = OtpMap::new(RwLock::new(HashMap::new()));

        Ok(State {
            config,

            database_pool,

            token_env,
            tokens,

            otps,
        })
    }
}
