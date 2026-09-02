use uuid::Uuid;
use warp::{
    Filter,
    reject::{self},
};

use crate::Error;

/// Authentication filter
fn with_auth() -> impl Filter<Extract = (), Error = warp::Rejection> + Copy {
    warp::cookie("admin-token")
        // Token is deserialized into `String` first, otherwise if it tried to
        // deserialize directly into `Uuid` and failed, it would reject with
        // `MissingCookie`, which is unwanted in this case.
        .and_then(async |token: String| -> Result<(), warp::Rejection> {
            let token = Uuid::try_parse(&token).map_err(|e| reject::custom(Error::from(e)))?;
            todo!()
        })
        .untuple_one()
}

pub fn all() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("api" / "admin" / ..).and(
        login().or(with_auth().and(
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
                .or(token_create())
                .or(token_update())
                .or(token_get()),
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
    warp::path!("asset" / Uuid)
        .and(warp::get())
        .map(|id| todo!())
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

/// Provision a new token
pub fn token_create() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("token" / "create")
        .and(warp::post())
        .map(|| todo!())
}

/// Update an admin token
pub fn token_update() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("token" / Uuid)
        .and(warp::put())
        .map(|id| todo!())
}

/// Get status of a token
pub fn token_get() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("token" / Uuid)
        .and(warp::get())
        .map(|id| todo!())
}

/// Login as an admin
pub fn login() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("login").and(warp::post()).map(|| todo!())
}
