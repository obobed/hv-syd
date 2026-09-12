use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Attendee, routes::json_body};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateAttendeeBody {
    id: Uuid,
    first_name: String,
    last_name: String,
}

/// Insert an attendee into the databse.
///
/// Note that there exists the possibility of an id conflict.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/attendee/create",
    request_body = CreateAttendeeBody,
    responses(
        (status = 200, description = "Successfully created attendee."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
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

            Ok(warp::reply())
        })
}

/// Get a list of attendees
///
/// List **ALL** attendees. Note that this will use more resources relative
/// to other endpoints. It is recommended that use of this endpoint is limited.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/attendee/list",
    responses(
        (status = 200, body = Vec<Attendee>, description = "A list of attendees is returned."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
struct UpdateAttendeeBody {
    #[serde(default, deserialize_with = "crate::models::double_option")]
    // Team name could be present but null:
    //
    // Some(Some(v)) => v,
    // Some(None) => null,
    // None => not present
    team_name: Option<Option<String>>,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    first_name: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    last_name: Option<String>,
}

/// Update attendee details
///
/// For each field in the request body, update the corresponding
/// field of the attendee. This requires that at least one of the expected
/// fields exist.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/attendee/{id}",
    params(("id" = Uuid, Path, description = "Id of the attendee to update")),
    request_body = UpdateAttendeeBody,
    responses(
        (status = 200, description = "Attendee's details updated successfully."),
        (status = 400, description = "All fields of request body were `null`."),
        (status = 404, description = "An attendee with the given `id` was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
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

            let mut separated = builder.separated(",");

            match body.team_name {
                Some(Some(team_name)) => {
                    separated.push("teamName=");
                    separated.push_bind_unseparated(team_name);
                }
                Some(None) => {
                    separated.push("teamName=NULL");
                }
                None => (),
            }
            if let Some(first_name) = body.first_name {
                separated.push("firstName=");
                separated.push_bind_unseparated(first_name);
            }
            if let Some(last_name) = body.last_name {
                separated.push("lastName=");
                separated.push_bind_unseparated(last_name);
            }

            builder.push(" WHERE id=");
            builder.push_bind(id);
            builder.push(";");

            builder.build().execute(&db).await?;

            Ok(warp::reply())
        })
}
