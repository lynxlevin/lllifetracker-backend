use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TagType {
    Ambition,
    DesiredState,
    Action,
    Plain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub tag_type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagCreateRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagUpdateRequest {
    pub name: String,
}
