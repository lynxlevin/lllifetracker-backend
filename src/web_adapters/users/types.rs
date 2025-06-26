#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserVisible {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfirmationToken {
    pub user_id: uuid::Uuid,
}

pub const USER_ID_KEY: &str = "user_id";
pub const USER_EMAIL_KEY: &str = "user_email";
