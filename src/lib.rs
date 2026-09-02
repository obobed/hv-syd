mod routes;

pub mod error;
pub use error::Error;

use warp::{self, Filter};

pub fn router() -> impl Filter<Extract = (impl warp::Reply,)> + Clone {
    routes::admin::all().recover(error::handle_rejection)
}
