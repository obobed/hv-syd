use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use warp::Filter;

use crate::{Error, routes::json_body};

type Milestones = Vec<Milestone>;

#[derive(Serialize, Deserialize)]
struct Milestone {
    name: String,
    description: String,
    requirements: Vec<String>,
    prizes: Vec<Prize>,
    optional: bool,
}

#[derive(Serialize, Deserialize)]
struct Prize {
    name: String,
    description: String,
    image: Option<String>,
    participant_notes: Option<Vec<String>>,
    stock: i64,
}

/// Set milestones and prizes
///
/// Note: This deletes all existing milestones and prizes!
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
                    "sql/insert_milestone.sql",
                    i,
                    milestone.description,
                    serde_json::json!(milestone.requirements),
                    milestone.optional
                )
                .execute(trans.as_mut())
                .await?;

                for prize in milestone.prizes {
                    sqlx::query_file!(
                        "sql/insert_prize.sql",
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
