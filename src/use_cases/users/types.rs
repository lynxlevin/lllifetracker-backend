#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserVisible {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
