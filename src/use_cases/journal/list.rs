use db_adapters::{
    diary_adapter::DiaryAdapter, reading_note_adapter::ReadingNoteAdapter,
    thinking_note_adapter::ThinkingNoteAdapter,
};
use entities::user as user_entity;

use crate::{
    journal::{
        diaries::list::list_diaries,
        reading_notes::list::list_reading_notes,
        thinking_notes::{list::list_thinking_notes, types::ThinkingNoteListQuery},
        types::JournalVisibleWithTags,
    },
    UseCaseError,
};

pub async fn list_journals<'a>(
    user: user_entity::Model,
    diary_adapter: DiaryAdapter<'a>,
    reading_note_adapter: ReadingNoteAdapter<'a>,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<Vec<JournalVisibleWithTags>, UseCaseError> {
    let diaries = list_diaries(user.clone(), diary_adapter).await?;
    let reading_notes = list_reading_notes(user.clone(), reading_note_adapter).await?;
    let thinking_notes = list_thinking_notes(
        user.clone(),
        ThinkingNoteListQuery {
            resolved: None,
            archived: Some(true),
        },
        thinking_note_adapter,
    )
    .await?;

    Err(UseCaseError::InternalServerError("test".to_string()))
    // Ok(res)
}
