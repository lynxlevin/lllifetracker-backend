use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
