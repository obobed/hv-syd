use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Submission, routes::json_body};

/// Fetch all submissions
///
/// Note that this lists **ALL** submissions and will use more resources relative
/// to other endpoints. It is recommended that use of this endpoint is limited.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/submission/list",
    responses(
        (status = 200, body = Vec<Submission>, description = "List of submissions is returned."),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn list(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / "list")
        .and(warp::get())
        .map(|| db.clone())
        .then(async |db| -> Result<_, Error> {
            trace!("list submissions (GET /api/admin/submission/list)");

            // This won't scale :3 Should I consider a limit?
            let res = sqlx::query_file_as!(Submission, "sql/submissions.sql")
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&res))
        })
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct UpdateSubmissionBody {
    milestone: Option<i64>,
    description: Option<String>,
    attachment: Option<String>,
    approved: Option<bool>,
}

/// Update a submission
///
/// For each field in the request body, update the corresponding
/// field of the submission. This requires that at least one of the expected
/// fields exist.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/submission/{id}",
    params(("id" = Uuid, Path, description = "Id of the submission to update")),
    request_body = UpdateSubmissionBody,
    responses(
        (status = 200, description = "Submission details updated successfully."),
        (status = 400, description = "All fields of request body were missing."),
        (status = 404, description = "An submission with the given `id` was not found."),
        (status = 500, description = "An internal database error occured.")
    ),
))]
pub fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("submission" / String)
        .and(warp::post())
        .map(|id| (id, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(async |id: String, db, body: UpdateSubmissionBody| {
            Uuid::try_parse(&id)?;
            trace!("update submission details (POST /api/admin/team/{})", id);

            sqlx::query_scalar!("SELECT 1 FROM submissions WHERE id = ?", &id)
                .fetch_one(&db)
                .await?;

            if body.milestone.is_none()
                && body.description.is_none()
                && body.attachment.is_none()
                && body.approved.is_none()
            {
                return Err(Error::BadRequest);
            }

            let mut builder = sqlx::QueryBuilder::new("UPDATE submissions SET ");

            if let Some(milestone) = body.milestone {
                builder.push("milestone = ");
                builder.push_bind(milestone);
            }
            if let Some(description) = body.description {
                builder.push("description = ");
                builder.push_bind(description);
            }
            if let Some(attachment) = body.attachment {
                builder.push("attachmentLink = ");
                builder.push_bind(attachment);
            }
            if let Some(approved) = body.approved {
                builder.push("approved = ");
                builder.push_bind(approved);
            }

            builder.push(" WHERE id = ");
            builder.push_bind(id);
            builder.push(";");

            builder.build().execute(&db).await?;

            Ok(warp::reply())
        })
}
