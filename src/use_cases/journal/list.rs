use std::collections::VecDeque;

use db_adapters::{
    diary_adapter::DiaryAdapter, reading_note_adapter::ReadingNoteAdapter,
    thinking_note_adapter::ThinkingNoteAdapter,
};
use entities::user as user_entity;

use crate::{
    journal::{
        diaries::{list::list_diaries, types::DiaryVisibleWithTags},
        reading_notes::{self, list::list_reading_notes, types::ReadingNoteVisibleWithTags},
        thinking_notes::{
            list::list_thinking_notes,
            types::{ThinkingNoteListQuery, ThinkingNoteVisibleWithTags},
        },
        types::{JournalListQuery, JournalVisibleWithTags},
    },
    UseCaseError,
};

pub async fn list_journals<'a>(
    user: user_entity::Model,
    query: JournalListQuery,
    diary_adapter: DiaryAdapter<'a>,
    reading_note_adapter: ReadingNoteAdapter<'a>,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<Vec<JournalVisibleWithTags>, UseCaseError> {
    let mut diaries = list_diaries(user.clone(), diary_adapter)
        .await?
        .into_iter()
        .collect::<VecDeque<_>>();
    let mut reading_notes = list_reading_notes(user.clone(), reading_note_adapter)
        .await?
        .into_iter()
        .collect::<VecDeque<_>>();
    let mut thinking_notes = list_thinking_notes(
        user.clone(),
        ThinkingNoteListQuery {
            resolved: None,
            archived: Some(false),
        },
        thinking_note_adapter,
    )
    .await?
    .into_iter()
    .collect::<VecDeque<_>>();

    let mut res = vec![];
    let count = diaries.len() + reading_notes.len() + thinking_notes.len();

    for _ in 0..count {
        let first_thinking_note_is_unresolved = thinking_notes
            .front()
            .is_some_and(|t| t.resolved_at.is_none());
        if first_thinking_note_is_unresolved {
            res.push(JournalVisibleWithTags::from(thinking_notes.pop_front()));
            continue;
        }

        let diary_remains = diaries.len() > 0;
        let reading_note_remains = reading_notes.len() > 0;
        let thinking_note_remains = thinking_notes.len() > 0;

        let first = match (diary_remains, reading_note_remains, thinking_note_remains) {
            (true, false, false) => JournalVisibleWithTags::from(diaries.pop_front()),
            (false, true, false) => JournalVisibleWithTags::from(reading_notes.pop_front()),
            (false, false, true) => JournalVisibleWithTags::from(thinking_notes.pop_front()),
            (true, true, false) => pop_diary_or_reading_note(&mut diaries, &mut reading_notes),
            (true, false, true) => pop_diary_or_thinking_note(&mut diaries, &mut thinking_notes),
            (false, true, true) => {
                pop_reading_note_or_thinking_note(&mut reading_notes, &mut thinking_notes)
            }
            (true, true, true) => {
                if diaries.front().unwrap().date >= reading_notes.front().unwrap().date {
                    pop_diary_or_thinking_note(&mut diaries, &mut thinking_notes)
                } else {
                    pop_reading_note_or_thinking_note(&mut reading_notes, &mut thinking_notes)
                }
            }
            (false, false, false) => unreachable!("This should not happen, (None, None, None)."),
        };
        res.push(first);
    }

    Ok(res)
}

// MYMEMO implement Journal trait and use those to merge these methods.
fn pop_diary_or_reading_note(
    diaries: &mut VecDeque<DiaryVisibleWithTags>,
    reading_notes: &mut VecDeque<ReadingNoteVisibleWithTags>,
) -> JournalVisibleWithTags {
    if diaries.front().unwrap().date >= reading_notes.front().unwrap().date {
        JournalVisibleWithTags::from(diaries.pop_front())
    } else {
        JournalVisibleWithTags::from(reading_notes.pop_front())
    }
}

fn pop_diary_or_thinking_note(
    diaries: &mut VecDeque<DiaryVisibleWithTags>,
    thinking_notes: &mut VecDeque<ThinkingNoteVisibleWithTags>,
) -> JournalVisibleWithTags {
    if diaries.front().unwrap().date >= thinking_notes.front().unwrap().updated_at.date_naive() {
        JournalVisibleWithTags::from(diaries.pop_front())
    } else {
        JournalVisibleWithTags::from(thinking_notes.pop_front())
    }
}

fn pop_reading_note_or_thinking_note(
    reading_notes: &mut VecDeque<ReadingNoteVisibleWithTags>,
    thinking_notes: &mut VecDeque<ThinkingNoteVisibleWithTags>,
) -> JournalVisibleWithTags {
    if reading_notes.front().unwrap().date
        >= thinking_notes.front().unwrap().updated_at.date_naive()
    {
        JournalVisibleWithTags::from(reading_notes.pop_front())
    } else {
        JournalVisibleWithTags::from(thinking_notes.pop_front())
    }
}
