use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    journal::{
        diaries::types::DiaryVisibleWithTags, reading_notes::types::ReadingNoteVisibleWithTags,
        thinking_notes::types::ThinkingNoteVisibleWithTags,
    },
    tags::types::TagVisible,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum JournalKind {
    Diary,
    ReadingNote,
    ThinkingNote,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JournalVisibleWithTags {
    pub diary: Option<DiaryVisibleWithTags>,
    pub reading_note: Option<ReadingNoteVisibleWithTags>,
    pub thinking_note: Option<ThinkingNoteVisibleWithTags>,
    pub kind: JournalKind,
}
pub trait IntoJournalVisibleWithTags {
    fn push_tag(&mut self, tag: TagVisible);
    fn sort_key(&self) -> NaiveDate;
    fn is_newer_or_eq<T: IntoJournalVisibleWithTags>(&self, other: &T) -> bool;
}
impl From<DiaryVisibleWithTags> for JournalVisibleWithTags {
    fn from(value: DiaryVisibleWithTags) -> Self {
        Self {
            diary: Some(value),
            reading_note: None,
            thinking_note: None,
            kind: JournalKind::Diary,
        }
    }
}
impl From<ReadingNoteVisibleWithTags> for JournalVisibleWithTags {
    fn from(value: ReadingNoteVisibleWithTags) -> Self {
        Self {
            diary: None,
            reading_note: Some(value),
            thinking_note: None,
            kind: JournalKind::ReadingNote,
        }
    }
}
impl From<ThinkingNoteVisibleWithTags> for JournalVisibleWithTags {
    fn from(value: ThinkingNoteVisibleWithTags) -> Self {
        Self {
            diary: None,
            reading_note: None,
            thinking_note: Some(value),
            kind: JournalKind::ThinkingNote,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct JournalListQuery {
    pub tag_id_or: Option<String>,
}
