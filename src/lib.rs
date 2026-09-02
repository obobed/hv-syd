mod routes;

pub mod error;

use warp::{self, Filter};

pub fn router() -> impl Filter<Extract = (impl warp::Reply,)> + Clone {
    routes::admin::all()
}
