use std::collections::VecDeque;

use futures::join;

use db_adapters::{
    diary_adapter::{DiaryAdapter, DiaryFilter, DiaryJoin, DiaryOrder, DiaryQuery},
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteJoin, ReadingNoteOrder, ReadingNoteQuery,
    },
    thinking_note_adapter::{
        ThinkingNoteAdapter, ThinkingNoteFilter, ThinkingNoteJoin, ThinkingNoteOrder, ThinkingNoteQuery,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{
    journal::{
        diaries::types::DiaryVisibleWithTags,
        reading_notes::types::ReadingNoteVisibleWithTags,
        thinking_notes::types::ThinkingNoteVisibleWithTags,
        types::{IntoJournalVisibleWithTags, JournalSearchRequest, JournalVisibleWithTags},
    },
    tags::types::TagVisible,
    UseCaseError,
};

pub async fn search_journals<'a>(
    user: user_entity::Model,
    params: JournalSearchRequest,
    diary_adapter: DiaryAdapter<'a>,
    reading_note_adapter: ReadingNoteAdapter<'a>,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<Vec<JournalVisibleWithTags>, UseCaseError> {
    let text_query = params.text.map_or(vec![], |text| {
        text.split([' ', '　']).map(|t| t.to_string()).collect::<Vec<_>>()
    });
    let tag_ids = params.tag_ids;
    let diaries_future = get_diaries(diary_adapter, &user, text_query.clone(), tag_ids.clone());
    let reading_notes_future = get_reading_notes(reading_note_adapter, &user, text_query.clone(), tag_ids.clone());
    let thinking_notes_future =
        get_thinking_notes(thinking_note_adapter, &user, text_query.clone(), tag_ids.clone());

    let (diaries, reading_notes, thinking_notes) =
        join!(diaries_future, reading_notes_future, thinking_notes_future);
    let mut diaries = diaries?.into_iter().collect::<VecDeque<_>>();
    let mut reading_notes = reading_notes?.into_iter().collect::<VecDeque<_>>();
    let mut thinking_notes = thinking_notes?.into_iter().collect::<VecDeque<_>>();

    let mut res = vec![];
    let count = diaries.len() + reading_notes.len() + thinking_notes.len();

    for _ in 0..count {
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

async fn get_diaries(
    diary_adapter: DiaryAdapter<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<DiaryVisibleWithTags>, UseCaseError> {
    let mut query = diary_adapter.join_tags().join_my_way_via_tags().filter_eq_user(user);
    if text_query.len() > 0 {
        query = query.filter_contains_texts(text_query);
    }
    if tag_ids.len() > 0 {
        query = query.filter_contains_tags(tag_ids);
    }
    let diaries = query
        .order_by_date(Desc)
        .order_by_id(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_direction_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<DiaryVisibleWithTags> = vec![];
    for diary in diaries {
        if res.is_empty() || res.last().unwrap().id != diary.id {
            let tags = match diary.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&diary)],
                None => vec![],
            };
            let res_diary = DiaryVisibleWithTags::from((diary, tags));
            res.push(res_diary);
        } else {
            if let Some(_) = diary.tag_id {
                res.last_mut().unwrap().push_tag(Into::<TagVisible>::into(&diary));
            }
        }
    }
    Ok(res)
}

async fn get_reading_notes(
    reading_note_adapter: ReadingNoteAdapter<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<ReadingNoteVisibleWithTags>, UseCaseError> {
    let mut query = reading_note_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(user);
    if text_query.len() > 0 {
        query = query.filter_contains_texts(text_query);
    }
    if tag_ids.len() > 0 {
        query = query.filter_contains_tags(tag_ids);
    }
    let reading_notes = query
        .order_by_date(Desc)
        .order_by_created_at(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_direction_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ReadingNoteVisibleWithTags> = vec![];
    for reading_note in reading_notes {
        if res.is_empty() || res.last().unwrap().id != reading_note.id {
            let tags = match reading_note.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&reading_note)],
                None => vec![],
            };
            let res_reading_note = ReadingNoteVisibleWithTags::from((reading_note, tags));
            res.push(res_reading_note);
        } else {
            if let Some(_) = reading_note.tag_id {
                res.last_mut()
                    .unwrap()
                    .push_tag(Into::<TagVisible>::into(&reading_note));
            }
        }
    }
    Ok(res)
}

async fn get_thinking_notes(
    thinking_note_adapter: ThinkingNoteAdapter<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<ThinkingNoteVisibleWithTags>, UseCaseError> {
    let mut query = thinking_note_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(user);
    if text_query.len() > 0 {
        query = query.filter_contains_texts(text_query);
    }
    if tag_ids.len() > 0 {
        query = query.filter_contains_tags(tag_ids);
    }
    let thinking_notes = query
        .order_by_resolved_at_nulls_first(Desc)
        .order_by_updated_at(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_direction_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ThinkingNoteVisibleWithTags> = vec![];
    for thinking_note in thinking_notes {
        if res.is_empty() || res.last().unwrap().id != thinking_note.id {
            let tags = match thinking_note.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&thinking_note)],
                None => vec![],
            };
            let res_thinking_note = ThinkingNoteVisibleWithTags::from((thinking_note, tags));
            res.push(res_thinking_note);
        } else {
            if let Some(_) = thinking_note.tag_id {
                res.last_mut()
                    .unwrap()
                    .push_tag(Into::<TagVisible>::into(&thinking_note));
            }
        }
    }
    Ok(res)
}
