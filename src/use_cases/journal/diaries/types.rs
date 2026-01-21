use db_adapters::diary_adapter::DiaryUpdateKey;
use entities::{diary, prelude::Diary};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::{journal::types::IntoJournalVisibleWithTags, tags::types::TagVisible};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiaryVisibleWithTags {
    pub id: uuid::Uuid,
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tags: Vec<TagVisible>,
}
impl From<(diary::Model, Vec<TagVisible>)> for DiaryVisibleWithTags {
    fn from(value: (diary::Model, Vec<TagVisible>)) -> Self {
        let (diary, tags) = value;
        Self {
            id: diary.id,
            text: diary.text,
            date: diary.date,
            tags,
        }
    }
}
impl IntoJournalVisibleWithTags for DiaryVisibleWithTags {
    fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }

    fn sort_key(&self) -> chrono::NaiveDate {
        self.date
    }

    fn is_newer_or_eq<T: IntoJournalVisibleWithTags>(&self, other: &T) -> bool {
        self.sort_key() >= other.sort_key()
    }
}

#[derive(Deserialize, Debug)]
pub struct DiaryListQuery {
    pub tag_id_or: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DiaryCreateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DiaryUpdateRequest {
    pub text: Option<String>,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub update_keys: Vec<DiaryUpdateKey>,
}
