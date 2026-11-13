use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, types::Json};
use warp::Filter;

use crate::Error;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct Milestone {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub requirements: Option<Json<Vec<String>>>,
    pub prizes: Json<Vec<String>>,
    pub optional: bool,
}

pub fn get(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("milestones")
        .and(warp::get())
        .map(|| db.clone())
        .then(async |db| -> Result<_, Error> {
            trace!("list milestones (GET /api/v1/milestones)");

            let query_result = sqlx::query_file_as!(Milestone, "sql/list-milestones.sql")
                .fetch_all(&db)
                .await?;

            Ok(warp::reply::json(&query_result))
        })
}
