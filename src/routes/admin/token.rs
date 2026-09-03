use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, reply::Reply};

use crate::{
    Error,
    models::{Level, TokenMetadata},
    routes::json_body,
    state::TokenMap,
};

#[derive(Serialize, Deserialize)]
pub struct CreateBody {
    pub level: Level,
    pub valid_for: TimeDelta,
}

/// Provision a new token
pub fn create(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / "create")
        .and(warp::post())
        .and(json_body())
        .map(|body: CreateBody| -> Result<String, Error> {
            log::trace!("create token (POST /api/admin/token/create)");

            let metadata = TokenMetadata::new(body.level, body.valid_for);
            let id = Uuid::new_v4().to_string();

            let mut wtxn = token_env.write_txn()?;
            tokens.put(&mut wtxn, &id, &metadata)?;
            wtxn.commit()?;

            Ok(id)
        })
}

#[derive(Serialize, Deserialize)]
struct UpdateBody {
    pub add: TimeDelta,
}

/// Update an admin token
pub fn update(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / String)
        .and(warp::put())
        .and(json_body())
        .map(|token: String, body: UpdateBody| {
            log::trace!("update token {} (POST /api/admin/token/{})", token, token);

            let token = Uuid::try_parse(&token)?.to_string();

            let rtxn = token_env.read_txn()?;
            let Some(metadata) = tokens.get(&rtxn, &token)? else {
                return Err(Error::NotFound);
            };
            rtxn.commit()?;

            let mut wtxn = token_env.write_txn()?;
            let ret = tokens.delete(&mut wtxn, &token);
            std::assert_matches!(ret, Ok(true));

            tokens.put(
                &mut wtxn,
                &token,
                &TokenMetadata {
                    exp: metadata.exp + body.add,
                    ..metadata
                },
            )?;
            wtxn.commit()?;

            Ok(warp::reply().into_response())
        })
}

/// Get status of a token
pub fn get(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / String)
        .and(warp::get())
        .map(|token: String| {
            log::trace!("get token {} (GET /api/admin/token/{})", token, token);

            let token = Uuid::try_parse(&token)?.to_string();

            let rtxn = token_env.read_txn()?;
            let Some(metadata) = tokens.get(&rtxn, &token)? else {
                return Err(Error::NotFound);
            };
            rtxn.commit()?;

            Ok(warp::reply::json(&metadata))
        })
}

/// List tokens and status
pub fn list(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / "list")
        .and(warp::get())
        .map(|| -> Result<warp::reply::Json, Error> {
            log::trace!("list tokens (GET /api/admin/token/list)");

            let rtxn = token_env.read_txn()?;
            let out: Vec<TokenMetadata> = tokens
                .iter(&rtxn)?
                .map(|v| v.map(|(_, v)| v))
                .collect::<Result<Vec<TokenMetadata>, _>>()?;
            rtxn.commit()?;

            Ok(warp::reply::json(&out))
        })
}
