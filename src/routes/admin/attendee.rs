use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, routes::json_body};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAttendeeBody {
    id: Uuid,
    first_name: String,
    last_name: String,
}

pub fn create(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("attendee" / "create")
        .and(warp::post())
        .map(|| db.clone())
        .and(json_body())
        .then(async |db, body: CreateAttendeeBody| -> Result<_, Error> {
            trace!("create attendee (POST /api/admin/attendee/create)");

            sqlx::query_file!(
                "sql/insert-attendee.sql",
                body.id.to_string(),
                body.first_name,
                body.last_name
            )
            .execute(&db)
            .await?;

            Ok(warp::reply::reply())
        })
}

#[derive(Serialize, Deserialize)]
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
        .then(async |db| -> Result<_, Error> {
            trace!("get attendee list (GET /api/admin/attendee/list)");

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
#[serde(rename_all = "camelCase")]
struct UpdateAttendeeBody {
    #[serde(default, deserialize_with = "crate::models::double_option")]
    // Team name could be present but null:
    //
    // Some(Some(v)) => v,
    // Some(None) => null,
    // None => not present
    team_name: Option<Option<String>>,
    first_name: Option<String>,
    last_name: Option<String>,
}

/// Update attendee details
pub fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("attendee" / String)
        .and(warp::post())
        .map(|id| (id, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(async |id: String, db, body: UpdateAttendeeBody| {
            Uuid::try_parse(&id)?;
            trace!("update attendee details (POST /api/admin/team/{})", id);

            sqlx::query_scalar!("SELECT 1 FROM attendees WHERE id = ?", &id)
                .fetch_one(&db)
                .await?;

            if body.team_name.is_none() && body.first_name.is_none() && body.last_name.is_none() {
                return Err(Error::BadRequest);
            }

            let mut builder = sqlx::QueryBuilder::new("UPDATE attendees SET ");

            match body.team_name {
                Some(Some(team_name)) => {
                    builder.push("teamName = ");
                    builder.push_bind(team_name);
                }
                Some(None) => {
                    builder.push("teamName = NULL");
                }
                None => (),
            }
            if let Some(first_name) = body.first_name {
                builder.push("firstName = ");
                builder.push_bind(first_name);
            }
            if let Some(last_name) = body.last_name {
                builder.push("lastName = ");
                builder.push_bind(last_name);
            }

            builder.push(" WHERE id = ");
            builder.push_bind(id);
            builder.push(";");

            builder.build().execute(&db).await?;

            Ok(warp::reply::reply())
        })
}
