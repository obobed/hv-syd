use rand::distr::SampleString as _;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, query_file, query_file_as, types::Json};
use warp::Filter;

use crate::{Error, routes::json_body, state::OtpMap};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
struct TeamCreateBody {
    team_name: String,
}
/// Create team
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/team/create",
    request_body = TeamCreateBody,
    responses(
        (status = 200, description = "Success."),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn create(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "create")
        .and(warp::post())
        .and(json_body())
        .and(warp::cookie("attendee-id"))
        .map(|body, id| (db.clone(), body, id))
        .untuple_one()
        .then(
            async |db, body: TeamCreateBody, id: String| -> Result<_, Error> {
                if sqlx::query_scalar!("SELECT 1 FROM teams WHERE teamName = ?", &body.team_name)
                    .fetch_one(&db)
                    .await
                    .is_ok()
                {
                    return Err(Error::Conflict);
                }

                let mut trans = db.begin().await?;

                query_file!("sql/insert-team.sql", &body.team_name)
                    .execute(trans.as_mut())
                    .await?;

                query_file!("sql/update-attendee-team.sql", &id, &body.team_name)
                    .execute(trans.as_mut())
                    .await?;

                trans.commit().await?;

                Ok(warp::reply::reply())
            },
        )
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct TeamDetails {
    team_name: String,
    #[cfg_attr(feature = "openapi", schema(value_type = Option<Vec<Attendee>>))]
    attendees: Option<Json<Vec<Attendee>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct Attendee {
    id: String,
    first_name: String,
    last_name: String,
}

/// Get attendee's team details
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/team/get",
    responses(
        (status = 200, body = TeamDetails, description = "Success."),
        (status = 404, description = "Attendee didn't have a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn get(
    db: &Pool<Sqlite>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "get")
        .and(warp::get())
        .and(warp::cookie("attendee-id"))
        .map(|id| (db.clone(), id))
        .untuple_one()
        .then(async |db, attendee_id: String| -> Result<_, Error> {
            let team = query_file!("sql/get-team-name.sql", &attendee_id)
                .fetch_one(&db)
                .await?;

            let team_name = team.name.ok_or(Error::NotFound)?;

            let team = query_file_as!(TeamDetails, "sql/team-details.sql", &team_name)
                .fetch_one(&db)
                .await?;

            Ok(warp::reply::json(&team))
        })
}

#[inline]
fn new_otp() -> String {
    return rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 4);
}

/// Get attendee's team details
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/team/otp",
    responses(
        (status = 200, body = String, description = "Success. Returns a 4-character OTP"),
        (status = 404, description = "Attendee didn't have a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn otp(
    db: &Pool<Sqlite>,
    otps: &OtpMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "otp")
        .and(warp::post())
        .and(warp::cookie("attendee-id"))
        .map(|id| (db.clone(), id))
        .untuple_one()
        .then(async |db, id: String| -> Result<_, Error> {
            let team = query_file!("sql/get-team-name.sql", &id)
                .fetch_one(&db)
                .await?;

            let team_name = team.name.ok_or(Error::NotFound)?;

            let otp = new_otp();

            otps.write()?.insert(team_name, otp.clone());

            Ok(otp)
        })
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct JoinBody {
    otp: String,
}

/// Get attendee's team details
#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/team/join",
    request_body = JoinBody,
    responses(
        (status = 200, description = "Success."),
        (status = 400, description = "Team has maximum members"),
        (status = 404, description = "Invalid OTP"),
        (status = 409, description = "Attendee already had a team"),
        (status = 500, description = "An internal database or IO error occured.")
    ),
))]
pub fn join(
    db: &Pool<Sqlite>,
    otps: &OtpMap,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("team" / "join")
        .and(warp::post())
        .and(warp::cookie("attendee-id"))
        .and(json_body())
        .map(|id, body| (db.clone(), id, body))
        .untuple_one()
        .then(async |db, id: String, body: JoinBody| -> Result<_, Error> {
            let existing_team = query_file!("sql/get-team-name.sql", &id)
                .fetch_one(&db)
                .await?;

            // Attendees *need* to contact an org to change teams, otherwise
            // people would probably abause this
            if existing_team.name.is_some() {
                return Err(Error::Conflict);
            }

            let new_team_name = {
                let guard = otps.read()?;
                guard.get(&body.otp).ok_or(Error::NotFound)?.to_owned()
            };

            let new_team = query_file_as!(TeamDetails, "sql/team-details.sql", &new_team_name)
                .fetch_one(&db)
                .await?;

            if new_team.attendees.is_some() && new_team.attendees.unwrap().len() >= 3 {
                return Err(Error::Custom(
                    400.try_into().unwrap(),
                    String::from("The maximum number of attendees in a team is 3."),
                ));
            }

            query_file!("sql/update-attendee-team.sql", &id, &new_team_name)
                .execute(&db)
                .await?;

            Ok(warp::reply::reply())
        })
}
