use warp::Filter;

mod markdown;

pub mod admin;
pub mod attendee;
pub mod session;

const JSON_MAX_LENGTH_BYTES: u64 = 1024 * 64;

pub(crate) fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Copy
where
    T: serde::de::DeserializeOwned + Send,
{
    warp::body::content_length_limit(JSON_MAX_LENGTH_BYTES).and(warp::body::json::<T>())
}

pub fn public_assets() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path(crate::PUBLIC_ASSET_FOLDER).and(warp::fs::dir(crate::PUBLIC_ASSET_FOLDER))
}

pub fn guides() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path(crate::GUIDES_FOLDER)
        .and(warp::fs::dir(crate::GUIDES_FOLDER))
        .then(markdown::handler)
}
