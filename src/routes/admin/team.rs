use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, types::Json};
use uuid::Uuid;
use warp::{Filter, reply::Reply as _};

use crate::{Error, routes::json_body};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Team {
    team_name: String,
}

/// Get a list of team names
pub fn list(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "list")
        .and(warp::get())
        .map(|| db.clone())
        .then(async |db| -> Result<warp::reply::Json, Error> {
            trace!("get teams list (GET /api/admin/team/list)");

            let response = sqlx::query_file_as!(Team, "sql/team_names.sql")
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&response))
        })
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct TeamDetails {
    team_name: String,
    attendees: Option<Json<Vec<Attendee>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Attendee {
    id: String,
    first_name: String,
    last_name: String,
}

/// Get team details
pub fn get(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / String)
        .and(warp::get())
        .map(|name| (name, db.clone()))
        .untuple_one()
        .then(async |name, db| -> Result<warp::reply::Json, Error> {
            trace!("get team details (GET /api/admin/team/{})", name);

            let response = sqlx::query_file_as!(TeamDetails, "sql/team_details.sql", name)
                .fetch_one(&db)
                .await?;

            Ok(warp::reply::json(&response))
        })
}

#[derive(Serialize, Deserialize)]
struct UpdateTeamBody {
    team_name: String,
}

/// Update attendee details
pub fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / String)
        .and(warp::put().or(warp::post()).unify())
        .map(|id| (id, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(
            async |name: String,
                   db,
                   body: UpdateTeamBody|
                   -> Result<warp::reply::Response, Error> {
                trace!("update team name (PUT/POST /api/admin/team/{})", name);

                _ = Uuid::try_parse(&name)?;

                _ = sqlx::query_scalar!("SELECT 1 FROM teams WHERE teamName = ?", &name)
                    .fetch_one(&db)
                    .await?;

                sqlx::query_file!("sql/update_team_name.sql", body.team_name, name)
                    .execute(&db)
                    .await?;

                Ok(warp::reply::reply().into_response())
            },
        )
}
