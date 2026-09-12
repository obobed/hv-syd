use log::trace;
use sqlx::{Pool, Sqlite};
use warp::Filter;

use crate::{Error, models::Milestone, routes::json_body};

type Milestones = Vec<Milestone>;

/// Set milestones and prizes
///
/// Note: This deletes all existing milestones and prizes!
#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/milestones/set",
    request_body = Vec<Milestone>,
    responses(
        (status = 200, description = "Successfully set milestones & prizes."),
        (status = 400, description = "Too many milestones (> 9223372036854775807)"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn set(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("milestones" / "set")
        .and(warp::put())
        .and(json_body())
        .then(async |body: Milestones| -> Result<_, Error> {
            trace!("set milestones (PUT /api/admin/milestones/set)");

            let mut trans = db.begin().await?;

            sqlx::query!("DELETE FROM milestones")
                .execute(trans.as_mut())
                .await?;
            sqlx::query!("DELETE FROM prizes")
                .execute(trans.as_mut())
                .await?;

            trans.commit().await?;

            let mut trans = db.begin().await?;

            for (i, milestone) in body.into_iter().enumerate() {
                let i = i64::try_from(i).map_err(|_| Error::BadRequest)?;

                sqlx::query_file!(
                    "sql/insert-milestone.sql",
                    i,
                    milestone.name,
                    milestone.description,
                    serde_json::json!(milestone.requirements),
                    milestone.optional
                )
                .execute(trans.as_mut())
                .await?;

                for prize in milestone.prizes {
                    sqlx::query_file!(
                        "sql/insert-prize.sql",
                        prize.name,
                        prize.description,
                        prize.image,
                        serde_json::json!(prize.participant_notes),
                        prize.stock,
                        i
                    )
                    .execute(trans.as_mut())
                    .await?;
                }
            }

            trans.commit().await?;

            Ok(warp::reply())
        })
}
