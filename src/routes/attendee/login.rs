use std::sync::LazyLock;

use chrono::{TimeDelta, Utc};
use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, routes::json_body};

static EXPIRY: LazyLock<String> = LazyLock::new(|| (Utc::now() + TimeDelta::days(2)).to_rfc2822());

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct LoginBody {
    /// Attend participant id.
    #[cfg_attr(feature = "openapi", schema(value_type = Uuid))]
    id: String,
}

/// Login as a participant.
///
/// Instructs the browser to set the `attendee-id` cookie. This simply
/// checks the participant exists.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/login",
    request_body = LoginBody,
    responses(
        (status = 200, description = "Successfully logged in."),
        (status = 404, description = "The attendee didn't exist."),
        (status = 500, description = "An internal database error occured.")
    ),
    security(()),
))]
pub fn login(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .map(|body| (body, db.clone()))
        .untuple_one()
        .then(async |body: LoginBody, db| -> Result<_, Error> {
            trace!("login with attend id (POST /api/v1/login");

            Uuid::try_parse(&body.id)?;

            // This is just a check to see whether the attendee exists
            sqlx::query_scalar!("SELECT 1 FROM attendees WHERE id = ?", &body.id)
                .fetch_one(&db)
                .await?;

            Ok(warp::reply::with_header(
                warp::reply(),
                "Set-Cookie",
                format!("attendee-id={}; Expires={}", body.id, &*EXPIRY),
            ))
        })
}
