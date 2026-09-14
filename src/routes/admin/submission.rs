use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use warp::Filter;

use crate::{Error, models::Submission, routes::json_body};

/// Fetch an overview of all? submissions
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
struct UpdateSubmissionBody {
    milestone: Option<i64>,
    description: Option<String>,
    attachment: Option<String>,
    approved: Option<bool>,
}

/// Update a submission
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
