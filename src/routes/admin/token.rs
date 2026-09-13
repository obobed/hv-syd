use chrono::TimeDelta;
use log::trace;
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateBody {
    pub level: Level,
    #[cfg_attr(feature = "openapi", schema(
        value_type = [i32; 2],
        max_items = 2,
        min_items = 2,
        example = json!([0, 1])
    ))]
    pub valid_for: TimeDelta,
}

/// Provision a new token
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/token/create",
    request_body = CreateBody,
    responses(
        (status = 200, body = Uuid, description = "The created token."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn create(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / "create")
        .and(warp::post())
        .and(json_body())
        .map(|body: CreateBody| -> Result<String, Error> {
            trace!("create token (POST /api/admin/token/create)");

            let metadata = TokenMetadata::new(body.level, body.valid_for);
            let id = Uuid::new_v4().to_string();

            let mut wtxn = token_env.write_txn()?;
            tokens.put(&mut wtxn, &id, &metadata)?;
            wtxn.commit()?;

            Ok(id)
        })
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct UpdateBody {
    #[cfg_attr(feature = "openapi", schema(
        value_type = [i32; 2],
        max_items = 2,
        min_items = 2,
        example = json!([0, 1])
    ))]
    pub add: TimeDelta,
}

/// Update an admin token
///
/// Note that this endpoint only allows you to move the expiry date.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/token/{token}",
    params(("token" = Uuid, Path, description = "The token to update")),
    request_body = UpdateBody,
    responses(
        (status = 200, description = "Token updated successfully."),
        (status = 400, description = "The new expiry date would overflow."),
        (status = 404, description = "Token was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn update(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / String)
        .and(warp::post())
        .and(json_body())
        .map(|token: String, body: UpdateBody| {
            trace!("update token {} (POST /api/admin/token/{})", token, token);

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
                    exp: metadata
                        .exp
                        .checked_add_signed(body.add)
                        .ok_or(Error::Custom(
                            warp::http::StatusCode::BAD_REQUEST,
                            "would overflow".into(),
                        ))?,
                    ..metadata
                },
            )?;
            wtxn.commit()?;

            Ok(warp::reply().into_response())
        })
}

/// Get status of a token
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/token/{token}",
    params(("token" = Uuid, Path, description = "The token to get status of")),
    responses(
        (status = 200, body = TokenMetadata, description = "Token metadata."),
        (status = 404, description = "Token was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn get(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / String)
        .and(warp::get())
        .map(|token: String| {
            trace!("get token {} (GET /api/admin/token/{})", token, token);

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
///
/// This returns an object, with its keys being tokens and values being
/// the associated token metadata.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/token/list",
    responses(
        (
            status = 200,
            body = std::collections::HashMap<Uuid, TokenMetadata>,
            description = "List of tokens and associated metadata."
        ),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn list(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / "list")
        .and(warp::get())
        .map(|| -> Result<warp::reply::Json, Error> {
            trace!("list tokens (GET /api/admin/token/list)");

            let rtxn = token_env.read_txn()?;
            let out = tokens
                .iter(&rtxn)?
                .map(|v| v.map(|(k, v)| (k.to_string(), v)))
                .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
            rtxn.commit()?;

            Ok(warp::reply::json(&out))
        })
}
