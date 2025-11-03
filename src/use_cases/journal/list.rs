use std::collections::VecDeque;

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
        types::{IntoJournalVisibleWithTags, JournalListQuery, JournalVisibleWithTags},
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
            res.push(thinking_notes.pop_front().unwrap().into());
            continue;
        }

        let diary_remains = diaries.len() > 0;
        let reading_note_remains = reading_notes.len() > 0;
        let thinking_note_remains = thinking_notes.len() > 0;

        let first = match (diary_remains, reading_note_remains, thinking_note_remains) {
            (true, false, false) => diaries.pop_front().unwrap().into(),
            (false, true, false) => reading_notes.pop_front().unwrap().into(),
            (false, false, true) => thinking_notes.pop_front().unwrap().into(),
            (true, true, false) => pop_front_from_newer(&mut diaries, &mut reading_notes),
            (true, false, true) => pop_front_from_newer(&mut diaries, &mut thinking_notes),
            (false, true, true) => pop_front_from_newer(&mut reading_notes, &mut thinking_notes),
            (true, true, true) => {
                if a_is_newer(&diaries, &reading_notes) {
                    pop_front_from_newer(&mut diaries, &mut thinking_notes)
                } else {
                    pop_front_from_newer(&mut reading_notes, &mut thinking_notes)
                }
            }
            (false, false, false) => unreachable!("This should not happen, (None, None, None)."),
        };
        res.push(first);
    }

    Ok(res)
}

fn a_is_newer<T: IntoJournalVisibleWithTags, U: IntoJournalVisibleWithTags>(
    a: &VecDeque<T>,
    b: &VecDeque<U>,
) -> bool {
    a.front().unwrap().is_newer_or_eq(b.front().unwrap())
}

fn pop_front_from_newer<
    T: IntoJournalVisibleWithTags + Into<JournalVisibleWithTags>,
    U: IntoJournalVisibleWithTags + Into<JournalVisibleWithTags>,
>(
    a: &mut VecDeque<T>,
    b: &mut VecDeque<U>,
) -> JournalVisibleWithTags {
    if a_is_newer(&a, &b) {
        a.pop_front().unwrap().into()
    } else {
        b.pop_front().unwrap().into()
    }
}
