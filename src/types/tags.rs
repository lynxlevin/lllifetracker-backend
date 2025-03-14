use sea_orm::FromQueryResult;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum TagType {
    Ambition,
    DesiredState,
    Action,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub tag_type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TagQueryResult {
    pub id: uuid::Uuid,
    pub ambition_name: Option<String>,
    pub desired_state_name: Option<String>,
    pub action_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
