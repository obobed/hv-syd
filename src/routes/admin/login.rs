use chrono::{TimeDelta, Utc};
use log::trace;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, reply::Reply as _};

use crate::{
    Error,
    models::{Level, TokenMetadata},
    routes::json_body,
    state::TokenMap,
};

#[derive(Serialize, Deserialize)]
struct CredentialBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
struct TokenBody {
    pub token: String,
}

/// Login as an admin using credentials
pub fn with_creds(
    username: &str,
    password: &str,
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .map(move |body: CredentialBody| {
            trace!("login with credentials (POST /api/admin/login)");

            if username != body.username || password != body.password {
                return Err(Error::Forbidden);
            }

            let metadata = TokenMetadata::new(Level::Admin, TimeDelta::hours(1));
            let token = Uuid::new_v4().to_string();

            let mut wtxn = token_env.write_txn()?;
            tokens.put(&mut wtxn, &token, &metadata)?;
            wtxn.commit()?;

            Ok(warp::reply::with_header(
                warp::reply(),
                "Set-Cookie",
                format!("admin-token={token}"),
            )
            .into_response())
        })
}

/// Login as an admin using token
pub fn with_token(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .map(|body: TokenBody| {
            trace!("login with token (POST /api/admin/login)");

            let token = Uuid::try_parse(&body.token)?.to_string();
            let rtxn = token_env.read_txn()?;
            let metadata = match tokens.get(&rtxn, &token)? {
                Some(m) => m,
                None => return Err(Error::Forbidden),
            };

            if metadata.level != Level::Admin || metadata.exp < Utc::now() {
                Err(Error::Forbidden)
            } else {
                Ok(warp::reply::with_header(
                    warp::reply(),
                    "Set-Cookie",
                    format!("admin-token={token}"),
                )
                .into_response())
            }
        })
}
