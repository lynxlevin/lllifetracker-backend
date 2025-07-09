use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserVisible {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub first_track_at: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
