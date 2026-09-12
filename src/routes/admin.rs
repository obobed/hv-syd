use chrono::Utc;
use log::trace;
use uuid::Uuid;
use warp::{Filter, reject};

use crate::{
    Error,
    models::Level,
    state::{State, TokenMap},
};

mod asset;
mod attendee;
mod login;
mod milestone;
mod prize;
mod submission;
mod team;
mod token;

/// Authentication filter
fn with_admin_auth(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::cookie("admin-token")
        // Token is deserialized into `String` first, otherwise if it tried to
        // deserialize directly into `Uuid` and failed, it would reject with
        // `MissingCookie`, which is unwanted in this case.
        .map(|token: String| -> Result<(), Error> {
            trace!("client claims token: {token}");

            let token = Uuid::try_parse(&token)?.to_string();
            let rtxn = token_env.read_txn()?;
            let Some(metadata) = tokens.get(&rtxn, &token)? else {
                return Err(Error::Forbidden);
            };

            if metadata.level != Level::Admin || metadata.exp < Utc::now() {
                Err(Error::Forbidden)
            } else {
                Ok(())
            }
        })
        .and_then(async |res: Result<(), Error>| res.map_err(reject::custom))
        .untuple_one()
}

/// All admin-related routes
pub fn all(
    state: &State,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let State {
        config,
        database_pool,
        token_env,
        tokens,
    } = state;

    warp::path!("api" / "admin" / ..).and(
        login::login(
            &config.admin.username,
            &config.admin.password,
            token_env,
            tokens,
        )
        .or(with_admin_auth(token_env, tokens).and(
            milestone::set(database_pool)
                .or(asset::create())
                .or(asset::delete())
                .or(asset::list())
                .or(asset::get())
                .or(submission::list(database_pool))
                .or(submission::update(database_pool))
                .or(attendee::create(database_pool))
                .or(attendee::list(database_pool))
                .or(attendee::update(database_pool))
                .or(team::list(database_pool))
                .or(team::update(database_pool))
                .or(team::get(database_pool))
                // Order here is important!! `token::list` MUST go on top,
                // otherwise /token/list will be interpreted as 'list' being
                // the token.
                .or(token::list(token_env, tokens))
                .or(token::create(token_env, tokens))
                .or(token::update(token_env, tokens))
                .or(token::get(token_env, tokens)),
        )),
    )
}

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Satellites' Platform Documentation",
        description = dedent::dedent!("
        Welcome to the platform documentation. These are the backend endpoints for
        admin actions.

        Every endpoint listed here requires authentication, except for `/login`.
        Use the admin credentials or a token in `/login`, then you should be able
        to use any of these endpoints with the cookie that is set."),
        contact(name = "valerie"),
        license(name = "AGPL-3.0-or-later", identifier = "AGPL-3.0-or-later")
    ),
    nest((path = "/api/admin", api = AdminApiDocInner))
)]
/// API Documentation for admin paths.
pub struct AdminApiDoc;

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        asset::create,
        asset::delete,
        asset::get,
        asset::list,
        attendee::list,
        attendee::update,
        login::login,
    ),
    security(("admin-token" = ["read:admin", "write:admin"])),
    modifiers(&SecurityDetails)
)]
/// This just holds all the paths so I can easily prefix them using `nest(...)`
struct AdminApiDocInner;

#[cfg(feature = "openapi")]
struct SecurityDetails;

#[cfg(feature = "openapi")]
impl utoipa::Modify for SecurityDetails {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

            components.add_security_scheme(
                "admin-token",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("admin-token"))),
            );
        } else {
            panic!("Failed to add api-key details: unable to get openapi components");
        }
    }
}
