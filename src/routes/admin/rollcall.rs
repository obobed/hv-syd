use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use warp::{Filter, reject, reply};

use crate::Error;

#[derive(FromRow, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct RollcallAttendee {
    id: String,
    first_name: String,
    last_name: String,
    team_name: Option<String>,
    #[sqlx(skip)]
    attend_registration_found: bool,
    #[sqlx(skip)]
    checked_in_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
struct Rollcall {
    attendance_available: bool,
    attendance_error_code: Option<AttendanceErrorCode>,
    attendees: Vec<RollcallAttendee>,
}

#[derive(Clone, Copy, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
enum AttendanceErrorCode {
    #[serde(rename = "ATTEND-01")]
    Configuration,
    #[serde(rename = "ATTEND-02")]
    RosterRequest,
}

#[derive(Clone)]
struct AttendClient {
    client: reqwest::Client,
    token: String,
    roster_url: reqwest::Url,
}

#[derive(Deserialize)]
struct AttendRoster {
    participants: Vec<AttendRegistration>,
}

#[derive(Deserialize)]
struct AttendRegistration {
    participant_event_id: Uuid,
    checked_in_at: Option<DateTime<Utc>>,
}

impl AttendClient {
    fn from_env() -> Option<Self> {
        let token = std::env::var("ATTEND_ROSTER_TOKEN").ok()?;
        if token.trim().is_empty() {
            return None;
        }
        let event = std::env::var("ATTEND_EVENT_ID").unwrap_or_else(|_| "haven-sydney".into());
        let mut roster_url =
            reqwest::Url::parse("https://attend.hackclub.com/api/v1/events").ok()?;
        roster_url
            .path_segments_mut()
            .ok()?
            .extend([event.as_str(), "participants", "roster"]);
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;
        Some(Self {
            client,
            token,
            roster_url,
        })
    }

    async fn roster(&self) -> Result<HashMap<Uuid, Option<DateTime<Utc>>>, reqwest::Error> {
        let roster: AttendRoster = self
            .client
            .get(self.roster_url.clone())
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(roster
            .participants
            .into_iter()
            .map(|registration| {
                (
                    registration.participant_event_id,
                    registration.checked_in_at,
                )
            })
            .collect())
    }
}

/// List the Haven roster and any matching self-check-ins from Attend.
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/rollcall",
    responses((status = 200, body = Rollcall)),
))]
pub fn list(
    db: &SqlitePool,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let attend = AttendClient::from_env();
    warp::path!("rollcall").and(warp::get()).and_then(move || {
        let db = db.clone();
        let attend = attend.clone();
        async move {
            let roster = read_rollcall(&db, attend.as_ref())
                .await
                .map_err(reject::custom)?;
            Ok::<_, warp::Rejection>(reply::with_header(
                reply::json(&roster),
                "Cache-Control",
                "no-store",
            ))
        }
    })
}

async fn read_rollcall(db: &SqlitePool, attend: Option<&AttendClient>) -> Result<Rollcall, Error> {
    let mut attendees = sqlx::query_as::<_, RollcallAttendee>(
        "SELECT id, firstName AS first_name, lastName AS last_name, teamName AS team_name
         FROM attendees ORDER BY firstName COLLATE NOCASE, lastName COLLATE NOCASE, id",
    )
    .fetch_all(db)
    .await?;

    let attendance = match attend {
        Some(client) => match client.roster().await {
            Ok(roster) => Ok(roster),
            Err(error) => {
                log::warn!(
                    "Could not load the Attend roster (HTTP status {:?})",
                    error.status()
                );
                Err(AttendanceErrorCode::RosterRequest)
            }
        },
        None => Err(AttendanceErrorCode::Configuration),
    };
    if let Ok(attendance) = &attendance {
        for attendee in &mut attendees {
            if let Some(check_in) = Uuid::parse_str(&attendee.id)
                .ok()
                .and_then(|id| attendance.get(&id))
            {
                attendee.attend_registration_found = true;
                attendee.checked_in_at = *check_in;
            }
        }
    }

    Ok(Rollcall {
        attendance_available: attendance.is_ok(),
        attendance_error_code: attendance.err(),
        attendees,
    })
}
