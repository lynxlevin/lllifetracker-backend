use crate::journal::{
    diaries::types::DiaryVisibleWithTags, reading_notes::types::ReadingNoteVisibleWithTags,
    thinking_notes::types::ThinkingNoteVisibleWithTags,
};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct JournalVisibleWithTags {
    pub diary: Option<DiaryVisibleWithTags>,
    pub reading_note: Option<ReadingNoteVisibleWithTags>,
    pub thinking_note: Option<ThinkingNoteVisibleWithTags>,
}
