use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, types::Json};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Team, routes::json_body};

/// Get a list of team names.
///
/// List all team names. This does not return any other data about teams.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/team/list",
    responses(
        (status = 200, body = Vec<Team>, description = "List of team names."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn list(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "list")
        .and(warp::get())
        .map(|| db.clone())
        .then(async |db| -> Result<_, Error> {
            trace!("get teams list (GET /api/admin/team/list)");

            let response = sqlx::query_file_as!(Team, "sql/team-names.sql")
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&response))
        })
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct TeamDetails {
    team_name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = Option<Vec<Attendee>>))]
    attendees: Option<Json<Vec<Attendee>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct Attendee {
    id: String,
    first_name: String,
    last_name: String,
}

/// Get team details
///
/// Get the details of one team.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/team/{name}",
    params(("name" = String, Path, description = "Name of the team to fetch details for")),
    responses(
        (status = 200, body = TeamDetails, description = "Team details."),
        (status = 404, description = "A team with the given `name` was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn get(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / String)
        .and(warp::get())
        .map(|name| (name, db.clone()))
        .untuple_one()
        .then(async |name, db| -> Result<_, Error> {
            trace!("get team details (GET /api/admin/team/{})", name);

            let response = sqlx::query_file_as!(TeamDetails, "sql/team-details.sql", name)
                .fetch_one(&db)
                .await?;

            Ok(warp::reply::json(&response))
        })
}

/// Update team details
///
/// Note that you can only update team names using this endpoint.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/team/{name}",
    params(("name" = String, Path, description = "Name of the team to update")),
    request_body = Team,
    responses(
        (status = 200, description = "Team name updated successfully."),
        (status = 404, description = "An team with the given `name` was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / String)
        .and(warp::post())
        .map(|id| (id, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(async |name: String, db, body: Team| -> Result<_, Error> {
            trace!("update team name (POST /api/admin/team/{})", name);

            Uuid::try_parse(&name)?;

            sqlx::query_scalar!("SELECT 1 FROM teams WHERE teamName = ?", &name)
                .fetch_one(&db)
                .await?;

            sqlx::query_file!("sql/update-team-name.sql", body.team_name, name)
                .execute(&db)
                .await?;

            Ok(warp::reply())
        })
}
