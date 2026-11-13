use log::trace;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::{Filter, reject};

use crate::{Error, state::State};

mod login;
mod milestones;
mod submission;
mod team;

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
                .or(submission::list(database_pool)),
        )),
    )
}
