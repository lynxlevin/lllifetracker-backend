use db_adapters::tag_adapter::TagWithNames;
use entities::sea_orm_active_enums::TagType as DBTagType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub r#type: DBTagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<TagWithNames> for TagVisible {
    fn from(value: TagWithNames) -> Self {
        let name = match value.r#type {
            DBTagType::Ambition => value.ambition_name,
            DBTagType::DesiredState => value.desired_state_name,
            DBTagType::Action => value.action_name,
            DBTagType::Plain => value.name,
        };

        if let None = name {
            panic!("This tag either has no name or type is incorrect");
        }

        Self {
            id: value.id,
            name: name.unwrap(),
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
