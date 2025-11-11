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
pub struct JournalVisibleWithTags {
    pub diary: Option<DiaryVisibleWithTags>,
    pub reading_note: Option<ReadingNoteVisibleWithTags>,
    pub thinking_note: Option<ThinkingNoteVisibleWithTags>,
}

pub trait IntoJournalVisibleWithTags {
    fn push_tag(&mut self, tag: TagVisible);
    fn sort_key(&self) -> NaiveDate;
    fn is_newer_or_eq<T: IntoJournalVisibleWithTags>(&self, other: &T) -> bool;
}

#[derive(Deserialize, Debug)]
pub struct JournalListQuery {
    pub tag_id_or: Option<String>,
}
