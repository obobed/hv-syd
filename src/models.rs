use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Attendee {
    #[cfg_attr(feature = "openapi", schema(value_type = Uuid))]
    pub id: String,
    pub team_name: Option<String>,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub prizes: Vec<Prize>,
    pub optional: bool,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Prize {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub participant_notes: Option<Vec<String>>,
    pub stock: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Submission {
    pub id: String,
    pub team_name: String,
    pub milestone: i64,
    pub description: Option<String>,
    pub attachment: Option<String>,
    pub approved: bool,
    /// Creation time, as a UNIX timestamp.
    pub creation_time: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub team_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenMetadata {
    pub level: Level,
    /// This is a timestamp
    pub exp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Admin = 0,
    Attendee = 1,
}

impl TokenMetadata {
    pub fn new(level: Level, valid_for: TimeDelta) -> TokenMetadata {
        TokenMetadata {
            level,
            exp: Utc::now() + valid_for,
        }
    }
}

pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}
