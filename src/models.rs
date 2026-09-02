use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TokenMetadata {
    pub level: Level,
    pub expiry: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum Level {
    Admin,
    Invite,
}
