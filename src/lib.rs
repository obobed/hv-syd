use crate::state::State;
use std::sync::Arc;
use warp::Filter;

mod config;
mod routes;
mod state;

pub mod error;

pub use config::Config;
pub use error::Error;

pub const PUBLIC_ASSET_FOLDER: &str = "assets";
pub const PRIVATE_ASSET_FOLDER: &str = "private";
pub const GUIDES_FOLDER: &str = "guides";

/// Literally every route
pub fn router(config: Config) -> impl Filter<Extract = (impl warp::Reply,)> + Clone {
    let state = Arc::new(State::new(config).expect("State::new"));

    routes::public_assets()
        .or(routes::guides())
        .or(routes::admin::all())
        .recover(error::handle_rejection)
}
