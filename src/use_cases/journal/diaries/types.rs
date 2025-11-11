use db_adapters::diary_adapter::DiaryUpdateKey;
use entities::{diary, prelude::Diary};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::{
    journal::types::{IntoJournalVisibleWithTags, JournalKind, JournalVisibleWithTags},
    tags::types::TagVisible,
};

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

impl Into<JournalVisibleWithTags> for DiaryVisibleWithTags {
    fn into(self) -> JournalVisibleWithTags {
        JournalVisibleWithTags {
            diary: Some(self),
            reading_note: None,
            thinking_note: None,
            kind: JournalKind::Diary,
        }
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
