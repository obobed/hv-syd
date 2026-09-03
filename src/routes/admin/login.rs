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
#[serde(rename_all = "snake_case", untagged)]
enum LoginBody {
    Credential { username: String, password: String },
    Token { token: String },
}

pub fn login(
    username: &str,
    password: &str,
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .map(move |body: LoginBody| match body {
            LoginBody::Credential {
                username: given_username,
                password: given_password,
            } => with_creds(
                username,
                password,
                &given_username,
                &given_password,
                token_env,
                tokens,
            )
            .into_response(),
            LoginBody::Token { token } => with_token(&token, token_env, tokens).into_response(),
        })
}

/// Login as an admin using credentials
fn with_creds(
    expected_username: &str,
    expected_password: &str,

    given_username: &str,
    given_password: &str,

    token_env: &heed::Env,
    tokens: &TokenMap,
) -> Result<impl warp::Reply, Error> {
    if expected_username != given_username || expected_password != given_password {
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
    ))
}

/// Login as an admin using token
fn with_token(
    token: &str,
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> Result<impl warp::Reply, Error> {
    trace!("login with token (POST /api/admin/login)");

    let token = Uuid::try_parse(token)?.to_string();
    let rtxn = token_env.read_txn()?;
    let Some(metadata) = tokens.get(&rtxn, &token)? else {
        return Err(Error::Forbidden);
    };

    if metadata.level != Level::Admin || metadata.exp < Utc::now() {
        Err(Error::Forbidden)
    } else {
        Ok(warp::reply::with_header(
            warp::reply(),
            "Set-Cookie",
            format!("admin-token={token}"),
        ))
    }
}
