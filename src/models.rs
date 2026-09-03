use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

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
    User = 1,
}

impl TokenMetadata {
    pub fn new(level: Level, valid_for: TimeDelta) -> TokenMetadata {
        TokenMetadata {
            level,
            exp: Utc::now() + valid_for,
        }
    }
}
