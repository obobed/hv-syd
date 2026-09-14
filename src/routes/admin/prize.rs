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
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/prize/add",
    request_body = Prize,
    responses(
        (status = 200, description = "Prize created"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn add(
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct IncrementPrizeBody {
    stock_change: i64,
}

/// "Update" a prize.
///
/// This only allows you to increment / decrement the stock count, given a prize
/// name.
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/prize/{name}",
    params(("name", Path, description = "The name of the prize to update")),
    request_body = IncrementPrizeBody,
    responses(
        (status = 200, description = "Prize updated"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn update(
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
