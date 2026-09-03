use crate::state::State;
use std::{sync::OnceLock, time::Duration};
use warp::Filter;

mod config;
mod models;
mod routes;
mod state;

pub mod error;

pub use config::Config;
pub use error::Error;

pub const PUBLIC_ASSET_FOLDER: &str = "assets";
pub const PRIVATE_ASSET_FOLDER: &str = "private";
pub const GUIDES_FOLDER: &str = "guides";
pub const TOKENS_STORE_FOLDER: &str = "tokens";

static STATE: OnceLock<State> = OnceLock::new();

/// Literally every route
pub fn router(config: Config) -> impl Filter<Extract = (impl warp::Reply,)> + Clone {
    let state = STATE.get_or_init(|| State::new(config).expect("State::new"));
    tokio::spawn(token_cleanup_task());

    routes::public_assets()
        .or(routes::guides())
        .or(routes::admin::all(state))
        .recover(error::handle_rejection)
}

async fn token_cleanup_task() {
    let state = STATE.get().expect("state was not initialized");

    loop {
        if let Err(e) = cleanup(state) {
            log::error!("cleanup failed: {}", e)
        }
        tokio::time::sleep(Duration::from_mins(5)).await;
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
