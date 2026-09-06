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
mod team;
mod token;

/// Authentication filter
fn with_admin_auth(
    token_env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::cookie("admin-token")
        .and(with_tokens(token_env, tokens))
        // Token is deserialized into `String` first, otherwise if it tried to
        // deserialize directly into `Uuid` and failed, it would reject with
        // `MissingCookie`, which is unwanted in this case.
        .map(|token: String, _env, _tokens| -> Result<(), Error> {
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

fn with_tokens(
    env: &heed::Env,
    tokens: &TokenMap,
) -> impl Filter<Extract = (heed::Env, TokenMap), Error = !> + Clone {
    warp::any().map(|| (env.clone(), *tokens)).untuple_one()
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
            milestones()
                .or(asset::create())
                .or(asset::delete())
                .or(asset::list())
                .or(asset::get())
                .or(submissions())
                .or(submission_get())
                .or(submission_update())
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

/// Modify milestones
pub fn milestones() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("milestones").and(warp::post()).map(|| todo!())
}

/// Fetch an overview of all? submissions
pub fn submissions() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("submissions").and(warp::get()).map(|| todo!())
}

/// Get a submission
pub fn submission_get()
-> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / Uuid)
        .and(warp::get())
        .map(|id| todo!())
}

/// Update a submission
pub fn submission_update()
-> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / Uuid)
        .and(warp::put())
        .map(|id| todo!())
}

/// Get a list of teams
pub fn teams() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("teams").and(warp::get()).map(|| todo!())
}

/// Update team details
pub fn team_update() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("team" / Uuid)
        .and(warp::put())
        .map(|id| todo!())
}

/// Get team details
pub fn team_get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / Uuid)
        .and(warp::get())
        .map(|id| todo!())
}
