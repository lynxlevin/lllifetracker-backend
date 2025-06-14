use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TagType {
    Ambition,
    DesiredState,
    Action,
    Plain,
}

#[derive(FromQueryResult, Debug, Serialize, Deserialize, PartialEq)]
pub struct TagQueryResult {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub ambition_name: Option<String>,
    pub desired_state_name: Option<String>,
    pub action_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub tag_type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<TagQueryResult> for TagVisible {
    fn from(item: TagQueryResult) -> Self {
        let (name, tag_type) = if let Some(name) = item.name {
            (name, TagType::Plain)
        } else if let Some(name) = item.ambition_name.clone() {
            (name, TagType::Ambition)
        } else if let Some(name) = item.desired_state_name.clone() {
            (name, TagType::DesiredState)
        } else if let Some(name) = item.action_name.clone() {
            (name, TagType::Action)
        } else {
            panic!("Tag without name should not exist.");
        };

        TagVisible {
            id: item.id,
            name,
            tag_type,
            created_at: item.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagCreateRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagUpdateRequest {
    pub name: String,
}
