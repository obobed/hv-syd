use chrono::Utc;
use log::trace;
use uuid::Uuid;
use warp::{Filter, reject};

use crate::{
    Error, PRIVATE_ASSET_FOLDER,
    models::Level,
    state::{State, TokenMap},
};

mod login;
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
                .or(asset_create())
                .or(asset_delete())
                .or(asset_get())
                .or(submissions())
                .or(submission_get())
                .or(submission_create())
                .or(submission_update())
                .or(users())
                .or(user_update())
                .or(user_get())
                .or(teams())
                .or(team_update())
                .or(team_get())
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

/// Create a public asset
pub fn asset_create() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("asset" / "create")
        .and(warp::post())
        .map(|| todo!())
}

/// Delete a public asset
pub fn asset_delete() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("asset" / Uuid)
        .and(warp::delete())
        .map(|id| todo!())
}

// Get a private asset
pub fn asset_get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("asset")
        .and(warp::get())
        .and(warp::fs::dir(PRIVATE_ASSET_FOLDER))
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

/// Create a submission
pub fn submission_create()
-> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / "create")
        .and(warp::post())
        .map(|| todo!())
}

/// Update a submission
pub fn submission_update()
-> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / Uuid)
        .and(warp::put())
        .map(|id| todo!())
}

/// Get a list of users
pub fn users() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("users").and(warp::get()).map(|| todo!())
}

/// Update user details
pub fn user_update() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("user" / Uuid)
        .and(warp::put())
        .map(|id| todo!())
}

/// Get user details
pub fn user_get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / Uuid)
        .and(warp::get())
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
