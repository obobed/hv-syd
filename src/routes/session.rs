use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use warp::Filter;

use crate::{Error, routes::json_body};

#[derive(Deserialize)]
struct AttendeeLogin {
    id: Uuid,
}

#[derive(Serialize)]
struct AttendeeSession {
    name: String,
}

pub fn attendee(
    db: &SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let login_db = db.clone();
    let session_db = db.clone();

    let login = warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .and_then(move |body: AttendeeLogin| {
            let db = login_db.clone();
            async move {
                if attendee_name(&db, body.id).await?.is_none() {
                    return Err(warp::reject::custom(Error::NotFound));
                }

                let cookie = format!(
                    "attendee-id={}; Path=/api/v1; Max-Age=172800; SameSite=Lax",
                    body.id
                );
                Ok(warp::reply::with_header(
                    warp::reply::with_header(warp::reply(), "Set-Cookie", cookie),
                    "Cache-Control",
                    "no-store",
                ))
            }
        });

    let session = warp::path!("session")
        .and(warp::get())
        .and(warp::cookie::<String>("attendee-id"))
        .and_then(move |cookie: String| {
            let db = session_db.clone();
            async move {
                let id =
                    Uuid::try_parse(&cookie).map_err(|_| warp::reject::custom(Error::Forbidden))?;
                let name = attendee_name(&db, id)
                    .await?
                    .ok_or_else(|| warp::reject::custom(Error::Forbidden))?;

                Ok::<_, warp::Rejection>(warp::reply::with_header(
                    warp::reply::json(&AttendeeSession { name }),
                    "Cache-Control",
                    "no-store",
                ))
            }
        });

    warp::path!("api" / "v1" / ..).and(login.or(session))
}

async fn attendee_name(db: &SqlitePool, id: Uuid) -> Result<Option<String>, warp::Rejection> {
    sqlx::query_scalar("SELECT firstName FROM attendees WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(db)
        .await
        .map_err(|error| warp::reject::custom(Error::from(error)))
}
