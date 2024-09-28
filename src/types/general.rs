#[derive(serde::Serialize, serde::Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub const USER_ID_KEY: &str = "user_id";
pub const USER_EMAIL_KEY: &str = "user_email";
pub const INTERNAL_SERVER_ERROR_MESSAGE: &str =
    "Some unexpected error happened. Please try again later.";
