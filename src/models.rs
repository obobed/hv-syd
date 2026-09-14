use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Deserializer, Serialize};

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
