use chrono::{DateTime, FixedOffset};
use entities::{prelude::ThinkingNote, thinking_note};
use sea_orm::{DerivePartialModel, FromQueryResult};
use uuid::Uuid;

use crate::tags::types::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
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

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
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

impl ThinkingNoteVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ThinkingNoteCreateRequest {
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub tag_ids: Vec<uuid::Uuid>,
}
