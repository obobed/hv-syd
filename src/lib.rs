use std::{fs, thread, time::Duration};
use warp::Filter;

use crate::state::State;

mod config;
mod models;
mod routes;
mod state;

pub mod error;

pub use config::Config;
pub use error::Error;

#[cfg(feature = "openapi")]
pub use routes::admin::AdminApiDoc;

pub const PUBLIC_ASSET_FOLDER: &str = "assets";
pub const PRIVATE_ASSET_FOLDER: &str = "private";
pub const GUIDES_FOLDER: &str = "guides";
pub const TOKENS_STORE_FOLDER: &str = "tokens";

/// Literally every route
pub fn router(config: Config) -> Result<impl Filter<Extract = (impl warp::Reply,)> + Clone, Error> {
    initialise_folders()?;

    let state = Box::leak(Box::new(State::new(config)?));

    initialise_db(state)?;
    thread::spawn(|| token_cleanup_task(state));

    Ok(routes::public_assets()
        .or(routes::guides())
        .or(routes::admin::all(state))
        .recover(error::handle_rejection))
}

fn token_cleanup_task(state: &State) {
    loop {
        if let Err(e) = cleanup(state) {
            log::error!("cleanup failed: {e}");
        }
        thread::sleep(Duration::from_mins(5));
    }
}

fn cleanup(state: &State) -> Result<(), Error> {
    let now = chrono::Utc::now();
    let mut wtxn = state.token_env.write_txn()?;
    let mut cursor = state.tokens.iter_mut(&mut wtxn)?;

    while let Some(entry) = cursor.next() {
        let (_k, v) = entry?;
        if v.exp < now {
            // SAFETY:
            //  - No references are stored
            unsafe {
                cursor.del_current()?;
            }
        }
    }
    drop(cursor);
    wtxn.commit()?;

    Ok(())
}

fn initialise_folders() -> Result<(), Error> {
    fs::create_dir_all(PUBLIC_ASSET_FOLDER)?;
    fs::create_dir_all(PRIVATE_ASSET_FOLDER)?;
    fs::create_dir_all(GUIDES_FOLDER)?;
    fs::create_dir_all(TOKENS_STORE_FOLDER)?;
    Ok(())
}

fn initialise_db(state: &State) -> Result<(), Error> {
    pollster::block_on(async {
        sqlx::query_file!("sql/create.sql")
            .execute(&state.database_pool)
            .await?;
        Ok(())
    })
}
