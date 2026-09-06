use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::{Filter, reply::Reply as _};

use crate::{Error, routes::json_body};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Attendee {
    id: String,
    team_name: Option<String>,
    first_name: String,
    last_name: String,
}

/// Get a list of users
pub fn list(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("attendee" / "list")
        .and(warp::get())
        .map(|| db.clone())
        .then(async |db| -> Result<warp::reply::Json, Error> {
            trace!("get attendee list (GET /api/admin/attendees)");

            // Note: WILL NOT SCALE!! But considering the platform will probably
            // handle 200 attendees max, this shouldn't be an issue :3
            //
            // Just don't spam this endpoint,,
            let response = sqlx::query_file_as!(Attendee, "sql/attendees.sql")
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&response))
        })
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct UpdateAttendeeBody {
    teamName: Option<String>,
    firstName: Option<String>,
    lastName: Option<String>,
}

/// Update attendee details
pub fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("attendee" / String)
        .and(warp::put().or(warp::post()).unify())
        .map(|id| (id, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(
            async |id: String,
                   db,
                   body: UpdateAttendeeBody|
                   -> Result<warp::reply::Response, Error> {
                _ = Uuid::try_parse(&id)?;
                trace!("update attendee details (PUT/POST /api/admin/team/{})", id);

                _ = sqlx::query_scalar!("SELECT 1 FROM attendees WHERE id = ?", &id)
                    .fetch_one(&db)
                    .await?;

                if body.teamName.is_none() && body.firstName.is_none() && body.lastName.is_none() {
                    return Err(Error::BadRequest);
                }

                let mut builder = sqlx::QueryBuilder::new("UPDATE attendees SET ");
                if let Some(team_name) = body.teamName {
                    builder.push("teamName = ");
                    builder.push_bind(team_name);
                }
                if let Some(first_name) = body.firstName {
                    builder.push("firstName = ");
                    builder.push_bind(first_name);
                }
                if let Some(last_name) = body.lastName {
                    builder.push("lastName = ");
                    builder.push_bind(last_name);
                }
                builder.push(" WHERE id = ");
                builder.push_bind(id);
                builder.push(";");

                builder.build().execute(&db).await?;

                Ok(warp::reply::reply().into_response())
            },
        )
}
