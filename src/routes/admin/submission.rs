use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Submission, routes::json_body};

#[derive(Serialize, sqlx::FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct AdminSubmission {
    #[serde(flatten)]
    #[sqlx(flatten)]
    submission: Submission,
    rejected: bool,
}

/// Fetch all submissions
///
/// Note that this lists **ALL** submissions and will use more resources relative
/// to other endpoints. It is recommended that use of this endpoint is limited.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/submission/list",
    responses(
        (status = 200, body = Vec<AdminSubmission>, description = "List of submissions is returned."),
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
            let res =
                sqlx::query_as::<_, AdminSubmission>(include_str!("../../../sql/submissions.sql"))
                    .fetch_all(&db)
                    .await?;

            Ok(warp::reply::json(&res))
        })
}

/// Fetch the submission and attachment reference for a staff review.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/submission/{id}",
    params(("id" = Uuid, Path, description = "ID of the submission to review")),
    responses(
        (status = 200, body = AdminSubmission, description = "Submission details."),
        (status = 400, description = "Invalid submission ID."),
        (status = 404, description = "Submission not found."),
        (status = 500, description = "An internal database error occurred."),
    ),
))]
pub fn get(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    warp::path!("submission" / String)
        .and(warp::get())
        .then(move |id: String| {
            let db = db.clone();
            async move {
                let id = Uuid::try_parse(&id)?.to_string();
                let submission = sqlx::query_as::<_, AdminSubmission>(
                    "SELECT id, teamName AS team_name, milestone, description, \
                            attachmentLink AS attachment, approved, rejected, creationTime AS creation_time \
                     FROM submissions WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(&db)
                .await?
                .ok_or(Error::NotFound)?;

                Ok::<_, Error>(warp::reply::json(&submission))
            }
        })
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct UpdateSubmissionBody {
    milestone: Option<i64>,
    description: Option<String>,
    attachment: Option<String>,
    approved: Option<bool>,
    status: Option<ReviewStatus>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
enum ReviewStatus {
    Unreviewed,
    Approved,
    Rejected,
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
            let id = Uuid::try_parse(&id)?.to_string();
            trace!(
                "update submission details (POST /api/admin/submission/{})",
                id
            );

            if body.status.is_some() && body.approved.is_some() {
                return Err(Error::BadRequest);
            }
            let status = body.status.or_else(|| {
                body.approved.map(|approved| {
                    if approved {
                        ReviewStatus::Approved
                    } else {
                        ReviewStatus::Unreviewed
                    }
                })
            });

            if body.milestone.is_none()
                && body.description.is_none()
                && body.attachment.is_none()
                && status.is_none()
            {
                return Err(Error::BadRequest);
            }

            let mut builder = sqlx::QueryBuilder::<Sqlite>::new("UPDATE submissions SET ");
            let mut fields = builder.separated(", ");

            if let Some(milestone) = body.milestone {
                fields.push("milestone = ").push_bind_unseparated(milestone);
            }
            if let Some(description) = body.description {
                fields
                    .push("description = ")
                    .push_bind_unseparated(description);
            }
            if let Some(attachment) = body.attachment {
                fields
                    .push("attachmentLink = ")
                    .push_bind_unseparated(attachment);
            }
            if let Some(status) = status {
                fields
                    .push("approved = ")
                    .push_bind_unseparated(matches!(status, ReviewStatus::Approved));
                fields
                    .push("rejected = ")
                    .push_bind_unseparated(matches!(status, ReviewStatus::Rejected));
            }

            let result = builder
                .push(" WHERE id = ")
                .push_bind(id)
                .build()
                .execute(&db)
                .await?;
            if result.rows_affected() == 0 {
                return Err(Error::NotFound);
            }

            Ok(warp::reply())
        })
}
