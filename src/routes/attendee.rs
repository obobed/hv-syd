use log::trace;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::{Filter, reject};

use crate::{Error, state::State};

mod login;
mod milestones;
mod submission;
mod team;
mod upload;

/// Authentication filter
fn with_attendee_auth(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::cookie("attendee-id")
        .map(|token| (db.clone(), token))
        .untuple_one()
        // Token is deserialized into `String` first, otherwise if it tried to
        // deserialize directly into `Uuid` and failed, it would reject with
        // `MissingCookie`, which is unwanted in this case.
        .then(async |db, token: String| -> Result<(), Error> {
            trace!("client claims token: {token}");

            let token = Uuid::try_parse(&token)?.to_string();

            // This is just a check to see whether the attendee exists
            if sqlx::query_scalar!("SELECT 1 FROM attendees WHERE id = ?", &token)
                .fetch_one(&db)
                .await
                .is_err()
            {
                return Err(Error::Forbidden);
            };

            Ok(())
        })
        .and_then(async |res: Result<(), Error>| res.map_err(reject::custom))
        .untuple_one()
}

/// All attendee-related routes
pub fn all(
    state: &State,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let State {
        config: _,
        database_pool,
        token_env: _,
        tokens: _,
        otps,
    } = state;

    warp::path!("api" / "v1" / ..).and(
        login::login(database_pool).or(with_attendee_auth(database_pool).and(
            milestones::get(database_pool)
                .or(team::create(database_pool))
                .or(team::get(database_pool))
                .or(team::otp(database_pool, otps))
                .or(team::join(database_pool, otps))
                .or(submission::create(database_pool))
                .or(submission::delete(database_pool))
                .or(submission::list(database_pool))
                .or(upload::upload(database_pool)),
        )),
    )
}

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Satellites' Platform Documentation",
        description = dedent::dedent!("
        Welcome to the platform documentation.

        Every endpoint listed here requires authentication, except for `/login`.
        Use the attend participant id in `/login`, then you should be able
        to use any of these endpoints with the cookie that is set.

        Note that error response variants documented are not exhaustive"),
        contact(name = "valerie"),
        license(name = "AGPL-3.0-or-later", identifier = "AGPL-3.0-or-later")
    ),
    nest((path = "/api/v1", api = AttendeeApiDocInner))
)]
/// API Documentation for admin paths.
pub struct AttendeeApiDoc;

#[cfg(feature = "openapi")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        login::login,
        milestones::get,
        team::create,
        team::get,
        team::otp,
        team::join,
        submission::list,
        submission::create,
        submission::delete,
        upload::upload
    ),
    security(("attendee-id" = ["read:attendee", "write:attendee"])),
    modifiers(&SecurityDetails)
)]
/// This just holds all the paths so I can easily prefix them using `nest(...)`
struct AttendeeApiDocInner;

#[cfg(feature = "openapi")]
struct SecurityDetails;

#[cfg(feature = "openapi")]
impl utoipa::Modify for SecurityDetails {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

            components.add_security_scheme(
                "attendee-id",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("attendee-id"))),
            );
        } else {
            panic!("Failed to add api-key details: unable to get openapi components");
        }
    }
}
