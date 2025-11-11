use entities::{prelude::ReadingNote, reading_note};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::{
    journal::types::{IntoJournalVisibleWithTags, JournalKind, JournalVisibleWithTags},
    tags::types::TagVisible,
};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "ReadingNote")]
pub struct ReadingNoteVisible {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<reading_note::Model> for ReadingNoteVisible {
    fn from(item: reading_note::Model) -> Self {
        ReadingNoteVisible {
            id: item.id,
            title: item.title,
            page_number: item.page_number,
            text: item.text,
            date: item.date,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReadingNoteVisibleWithTags {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tags: Vec<TagVisible>,
}

impl IntoJournalVisibleWithTags for ReadingNoteVisibleWithTags {
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

impl Into<JournalVisibleWithTags> for ReadingNoteVisibleWithTags {
    fn into(self) -> JournalVisibleWithTags {
        JournalVisibleWithTags {
            diary: None,
            reading_note: Some(self),
            thinking_note: None,
            kind: JournalKind::ReadingNote,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ReadingNoteListQuery {
    pub tag_id_or: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ReadingNoteCreateRequest {
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ReadingNoteUpdateRequest {
    pub title: Option<String>,
    pub page_number: Option<i16>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
}
