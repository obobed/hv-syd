use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use warp::Filter;

use crate::{Error, models::Prize, routes::json_body};

/// Add a prize.
///
/// Note that this is not the preferred way of adding prizes. This does not
/// allow the added prize to be associated with any milestone, and when
/// milestones are set, prizes are reset. See `/api/admin/milestone/set`
/// instead.
fn add(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("prize" / "add")
        .and(warp::post())
        .map(|| db.clone())
        .and(json_body())
        .then(async |db, body: Prize| -> Result<_, Error> {
            sqlx::query_file!(
                "sql/insert-prize.sql",
                body.name,
                body.description,
                body.image,
                serde_json::json!(body.participant_notes),
                body.stock,
                Option::<i64>::None
            )
            .execute(&db)
            .await?;

            Ok(warp::reply())
        })
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncrementPrizeBody {
    stock_change: i64,
}

/// "Update" a prize.
///
/// This only allows you to increment / decrement the stock count, given a prize
/// name.
fn update(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("prize" / String)
        .and(warp::post())
        .map(|s| (s, db.clone()))
        .untuple_one()
        .and(json_body())
        .then(
            async |name, db, body: IncrementPrizeBody| -> Result<_, Error> {
                sqlx::query_file!("sql/increment-prize.sql", body.stock_change, name,)
                    .execute(&db)
                    .await?;

                Ok(warp::reply())
            },
        )
}
