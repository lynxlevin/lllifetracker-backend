use sea_orm::FromQueryResult;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum TagType {
    Ambition,
    DesiredState,
    Action,
    Plain,
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TagQueryResult {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub ambition_name: Option<String>,
    pub desired_state_name: Option<String>,
    pub action_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub tag_type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<TagQueryResult> for TagVisible {
    fn from(item: TagQueryResult) -> Self {
        if let Some(name) = item.name {
            TagVisible {
                id: item.id,
                name,
                tag_type: TagType::Plain,
                created_at: item.created_at,
            }
        } else if let Some(name) = item.ambition_name.clone() {
            TagVisible {
                id: item.id,
                name,
                tag_type: TagType::Ambition,
                created_at: item.created_at,
            }
        } else if let Some(name) = item.desired_state_name.clone() {
            TagVisible {
                id: item.id,
                name,
                tag_type: TagType::DesiredState,
                created_at: item.created_at,
            }
        } else if let Some(name) = item.action_name.clone() {
            TagVisible {
                id: item.id,
                name,
                tag_type: TagType::Action,
                created_at: item.created_at,
            }
        } else {
            unimplemented!("Tag without name cannot be used.");
        }
    }
}
