use chrono::{DateTime, FixedOffset};
use entities::{prelude::ThinkingNote, thinking_note};
use sea_orm::{DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{journal::types::IntoJournalVisibleWithTags, tags::types::TagVisible};

#[derive(Serialize, Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "ThinkingNote")]
pub struct ThinkingNoteVisible {
    pub id: Uuid,
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
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
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tags: Vec<TagVisible>,
}
impl From<(thinking_note::Model, Vec<TagVisible>)> for ThinkingNoteVisibleWithTags {
    fn from(value: (thinking_note::Model, Vec<TagVisible>)) -> Self {
        let (thinking_note, tags) = value;
        Self {
            id: thinking_note.id,
            question: thinking_note.question,
            thought: thinking_note.thought,
            answer: thinking_note.answer,
            resolved_at: thinking_note.resolved_at,
            created_at: thinking_note.created_at,
            updated_at: thinking_note.updated_at,
            tags,
        }
    }
}
impl IntoJournalVisibleWithTags for ThinkingNoteVisibleWithTags {
    fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }

    fn sort_key(&self) -> chrono::NaiveDate {
        match self.resolved_at {
            Some(datetime) => datetime.date_naive(),
            None => self.updated_at.date_naive(),
        }
    }

    fn is_newer_or_eq<T: IntoJournalVisibleWithTags>(&self, other: &T) -> bool {
        self.sort_key() >= other.sort_key()
    }
}

#[derive(Deserialize, Debug)]
pub struct ThinkingNoteListQuery {
    pub resolved: Option<bool>,
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
}
