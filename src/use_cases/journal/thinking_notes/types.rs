use chrono::{DateTime, FixedOffset};
use entities::{prelude::ThinkingNote, thinking_note};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    journal::types::{IntoJournalVisibleWithTags, JournalVisibleWithTags},
    tags::types::TagVisible,
};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "ThinkingNote")]
pub struct ThinkingNoteVisible {
    pub id: Uuid,
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub archived_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<thinking_note::Model> for ThinkingNoteVisible {
    fn from(item: thinking_note::Model) -> Self {
        ThinkingNoteVisible {
            id: item.id,
            question: item.question,
            thought: item.thought,
            answer: item.answer,
            resolved_at: item.resolved_at,
            archived_at: item.archived_at,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ThinkingNoteVisibleWithTags {
    pub id: Uuid,
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub archived_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tags: Vec<TagVisible>,
}

impl IntoJournalVisibleWithTags for ThinkingNoteVisibleWithTags {
    fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }

    fn sort_key(&self) -> chrono::NaiveDate {
        self.updated_at.date_naive()
    }

    fn is_newer_or_eq<T: IntoJournalVisibleWithTags>(&self, other: &T) -> bool {
        self.sort_key() >= other.sort_key()
    }
}

impl Into<JournalVisibleWithTags> for ThinkingNoteVisibleWithTags {
    fn into(self) -> JournalVisibleWithTags {
        JournalVisibleWithTags {
            diary: None,
            reading_note: None,
            thinking_note: Some(self),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ThinkingNoteListQuery {
    pub resolved: Option<bool>,
    pub archived: Option<bool>,
    pub tag_id_or: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ThinkingNoteCreateRequest {
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ThinkingNoteUpdateRequest {
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub tag_ids: Vec<uuid::Uuid>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub archived_at: Option<DateTime<FixedOffset>>,
}
