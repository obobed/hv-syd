use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;
use warp::Filter;

use crate::Error;

#[derive(Serialize)]
struct AttendeeSession {
    name: String,
}

pub fn attendee(
    db: &SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();

    warp::path!("api" / "v1" / "session")
        .and(warp::get())
        .and(warp::cookie::<String>("attendee-id"))
        .and_then(move |cookie: String| {
            let db = db.clone();
            async move {
                let id =
                    Uuid::try_parse(&cookie).map_err(|_| warp::reject::custom(Error::Forbidden))?;
                let name: String =
                    sqlx::query_scalar("SELECT firstName FROM attendees WHERE id = ?")
                        .bind(id.to_string())
                        .fetch_optional(&db)
                        .await
                        .map_err(|error| warp::reject::custom(Error::from(error)))?
                        .ok_or_else(|| warp::reject::custom(Error::Forbidden))?;

                Ok::<_, warp::Rejection>(warp::reply::with_header(
                    warp::reply::json(&AttendeeSession { name }),
                    "Cache-Control",
                    "no-store",
                ))
            }
        })
}
