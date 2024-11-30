#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum TagType {
    Ambition,
    Objective,
    Action,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct TagVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub tag_type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
