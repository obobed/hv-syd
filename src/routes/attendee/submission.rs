use chrono::Utc;
use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, query_file};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Submission, routes::json_body};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct CreateSubmissionBody {
    milestone: i16,
    description: Option<String>,
    attachment: Option<String>,
}

/// Create submission for the attendee's team.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/submission/create",
    request_body = CreateSubmissionBody,
    responses(
        (status = 200, body = Uuid, description = "Successfully created submission, returns submission id."),
        (status = 404, description = "Attendee didn't have a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn create(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / "create")
        .and(warp::post())
        .and(warp::cookie("attendee-id"))
        .and(json_body())
        .map(|id, body| (db.clone(), id, body))
        .untuple_one()
        .then(
            async |db, id: String, body: CreateSubmissionBody| -> Result<_, Error> {
                let team = query_file!("sql/get-team-name.sql", &id)
                    .fetch_one(&db)
                    .await?;

                let team_name = team.name.ok_or(Error::NotFound)?;

                let sub_id = Uuid::new_v4().to_string();
                let creation_time = Utc::now().timestamp();

                query_file!(
                    "sql/insert-submission.sql",
                    sub_id,
                    team_name,
                    body.milestone,
                    body.description,
                    body.attachment,
                    false,
                    creation_time
                )
                .execute(&db)
                .await?;

                return Ok(sub_id);
            },
        )
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct DeleteSubmissionBody {
    /// Submission ID
    #[cfg_attr(feature = "openapi", schema(value_type = Uuid))]
    id: String,
}

/// Delete a submission.
#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/submission/delete",
    request_body = CreateSubmissionBody,
    responses(
        (status = 200, description = "Successfully deleted submission."),
        (status = 403, description = "Submission wasn't owned by attendee's team."),
        (status = 404, description = "Attendee didn't have a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn delete(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / "delete")
        .and(warp::put())
        .and(warp::cookie("attendee-id"))
        .and(json_body())
        .map(|id, body| (db.clone(), id, body))
        .untuple_one()
        .then(
            async |db, id: String, body: DeleteSubmissionBody| -> Result<_, Error> {
                let team = query_file!("sql/get-team-name.sql", &id)
                    .fetch_one(&db)
                    .await?;

                let team_name = team.name.ok_or(Error::NotFound)?;

                let submission = query_file!("sql/submission-by-id.sql", body.id)
                    .fetch_one(&db)
                    .await?;

                // Attendees should not be able to delete already approved submissions.
                if submission.team_name != team_name || submission.approved {
                    return Err(Error::Unauthorized);
                }

                query_file!("sql/delete-submission.sql", &id)
                    .execute(&db)
                    .await?;

                return Ok(warp::reply::reply());
            },
        )
}

/// Get milestones
///
/// Get milestones
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/submission/list",
    responses(
        (status = 200, body = Vec<Submission>, description = "Successfully listed submissions."),
        (status = 404, description = "Attendee didn't have a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn list(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / "list")
        .and(warp::get())
        .and(warp::cookie("attendee-id"))
        .map(|id| (db.clone(), id))
        .untuple_one()
        .then(async |db, id: String| -> Result<_, Error> {
            trace!("list submissions (GET /api/v1/submission/list)");

            let team = query_file!("sql/get-team-name.sql", &id)
                .fetch_one(&db)
                .await?;

            let team_name = team.name.ok_or(Error::NotFound)?;

            let res = sqlx::query_file_as!(Submission, "sql/submissions-by-team.sql", team_name)
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&res))
        })
}
