use serde::{Deserialize, Serialize};

use crate::journal::{
    diaries::types::DiaryVisibleWithTags, reading_notes::types::ReadingNoteVisibleWithTags,
    thinking_notes::types::ThinkingNoteVisibleWithTags,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct JournalVisibleWithTags {
    pub diary: Option<DiaryVisibleWithTags>,
    pub reading_note: Option<ReadingNoteVisibleWithTags>,
    pub thinking_note: Option<ThinkingNoteVisibleWithTags>,
}

impl From<Option<DiaryVisibleWithTags>> for JournalVisibleWithTags {
    fn from(value: Option<DiaryVisibleWithTags>) -> Self {
        Self {
            diary: value,
            reading_note: None,
            thinking_note: None,
        }
    }
}
impl From<Option<ReadingNoteVisibleWithTags>> for JournalVisibleWithTags {
    fn from(value: Option<ReadingNoteVisibleWithTags>) -> Self {
        Self {
            diary: None,
            reading_note: value,
            thinking_note: None,
        }
    }
}
impl From<Option<ThinkingNoteVisibleWithTags>> for JournalVisibleWithTags {
    fn from(value: Option<ThinkingNoteVisibleWithTags>) -> Self {
        Self {
            diary: None,
            reading_note: None,
            thinking_note: value,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct JournalListQuery {
    pub tag_id_or: Option<String>,
}
