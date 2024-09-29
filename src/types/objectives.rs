#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectiveVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
