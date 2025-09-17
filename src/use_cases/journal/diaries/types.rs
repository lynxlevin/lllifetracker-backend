use db_adapters::diary_adapter::DiaryUpdateKey;
use entities::{diary, prelude::Diary};
use sea_orm::{DerivePartialModel, FromQueryResult};

use crate::tags::types::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
#[sea_orm(entity = "Diary")]
pub struct DiaryVisible {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
}

impl From<diary::Model> for DiaryVisible {
    fn from(item: diary::Model) -> Self {
        DiaryVisible {
            id: item.id,
            text: item.text,
            date: item.date,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct DiaryVisibleWithTags {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tags: Vec<TagVisible>,
}

impl DiaryVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DiaryCreateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct DiaryUpdateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub update_keys: Vec<DiaryUpdateKey>,
}
