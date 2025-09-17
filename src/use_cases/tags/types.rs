use db_adapters::tag_adapter::TagWithName;
use entities::sea_orm_active_enums::TagType as DBTagType;
use serde::{Deserialize, Serialize};

// MYMEMO: same as TagWithName
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub r#type: DBTagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<TagWithName> for TagVisible {
    fn from(value: TagWithName) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
            created_at: value.created_at,
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
