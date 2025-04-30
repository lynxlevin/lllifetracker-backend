use entities::{diary, prelude::Diary};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Diary")]
pub struct DiaryVisible {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
}

impl From<diary::Model> for DiaryVisible {
    fn from(item: diary::Model) -> Self {
        DiaryVisible {
            id: item.id,
            text: item.text,
            date: item.date,
            score: item.score,
        }
    }
}

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DiaryWithTagQueryResult {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_id: Option<uuid::Uuid>,
    pub tag_name: Option<String>,
    pub tag_ambition_name: Option<String>,
    pub tag_desired_state_name: Option<String>,
    pub tag_mindset_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct DiaryVisibleWithTags {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tags: Vec<TagVisible>,
}

impl DiaryVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone, PartialEq)]
pub enum DiaryKey {
    Text,
    Date,
    Score,
    TagIds,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DiaryCreateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DiaryUpdateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<uuid::Uuid>,
    pub update_keys: Vec<DiaryKey>,
}
