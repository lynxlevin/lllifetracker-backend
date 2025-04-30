use entities::{prelude::ReadingNote, reading_note};
use sea_orm::{DerivePartialModel, FromQueryResult};

use super::TagVisible;

#[derive(
    serde::Serialize, serde::Deserialize, DerivePartialModel, FromQueryResult, PartialEq, Debug,
)]
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

#[derive(FromQueryResult, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ReadingNoteWithTagQueryResult {
    pub id: uuid::Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub tag_id: Option<uuid::Uuid>,
    pub tag_name: Option<String>,
    pub tag_ambition_name: Option<String>,
    pub tag_desired_state_name: Option<String>,
    pub tag_mindset_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
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

impl ReadingNoteVisibleWithTags {
    pub fn push_tag(&mut self, tag: TagVisible) {
        self.tags.push(tag);
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ReadingNoteCreateRequest {
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct ReadingNoteUpdateRequest {
    pub title: Option<String>,
    pub page_number: Option<i16>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
}
