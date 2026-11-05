use std::collections::VecDeque;

use domain_services::{
    diary_service::{DiaryService, DiaryServiceError, DiaryServiceQuery},
    reading_note_service::{ReadingNoteService, ReadingNoteServiceError, ReadingNoteServiceQuery},
    thinking_note_service::{ThinkingNoteService, ThinkingNoteServiceError, ThinkingNoteServiceQuery},
};
use futures::join;

use entities::user as user_entity;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    journal::{
        diaries::types::DiaryVisibleWithTags,
        reading_notes::types::ReadingNoteVisibleWithTags,
        thinking_notes::types::ThinkingNoteVisibleWithTags,
        types::{IntoJournalVisibleWithTags, JournalSearchRequest, JournalVisibleWithTags},
    },
    tags::types::TagVisible,
};

#[derive(Debug, Error)]
pub enum SearchJournalError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<DiaryServiceError> for SearchJournalError {
    fn from(e: DiaryServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}
impl From<ReadingNoteServiceError> for SearchJournalError {
    fn from(e: ReadingNoteServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}
impl From<ThinkingNoteServiceError> for SearchJournalError {
    fn from(e: ThinkingNoteServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.text.is_some = params.text.is_some(),
        params.tag_ids.len = params.tag_ids.len(),
    ),
    skip_all
)]
pub async fn search_journals<'a>(
    user: user_entity::Model,
    params: JournalSearchRequest,
    diary_service: DiaryService<'a>,
    reading_note_service: ReadingNoteService<'a>,
    thinking_note_service: ThinkingNoteService<'a>,
) -> Result<Vec<JournalVisibleWithTags>, SearchJournalError> {
    let text_query = params.text.map_or(vec![], |text| {
        text.split([' ', '　']).map(|t| t.to_string()).collect::<Vec<_>>()
    });
    let tag_ids = params.tag_ids;
    let diaries_future = get_diaries(diary_service, &user, text_query.clone(), tag_ids.clone());
    let reading_notes_future = get_reading_notes(reading_note_service, &user, text_query.clone(), tag_ids.clone());
    let thinking_notes_future =
        get_thinking_notes(thinking_note_service, &user, text_query.clone(), tag_ids.clone());

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
    diary_service: DiaryService<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<DiaryVisibleWithTags>, SearchJournalError> {
    let diaries = diary_service.search(user.id, text_query, tag_ids).await?;

    let mut res: Vec<DiaryVisibleWithTags> = vec![];
    for diary in diaries {
        if res.is_empty() || res.last().unwrap().id != diary.id {
            let tags = match diary.tag_id {
                Some(_) => {
                    let diary_ = diary.clone();
                    vec![TagVisible {
                        id: diary_.tag_id.unwrap(),
                        name: diary_.tag_name.unwrap(),
                        r#type: diary_.tag_type.unwrap(),
                        created_at: diary_.tag_created_at.unwrap(),
                    }]
                }
                None => vec![],
            };
            let res_diary = DiaryVisibleWithTags::from((diary, tags));
            res.push(res_diary);
        } else {
            if let Some(_) = diary.tag_id {
                res.last_mut().unwrap().push_tag(TagVisible {
                    id: diary.tag_id.unwrap(),
                    name: diary.tag_name.unwrap(),
                    r#type: diary.tag_type.unwrap(),
                    created_at: diary.tag_created_at.unwrap(),
                });
            }
        }
    }
    Ok(res)
}

async fn get_reading_notes(
    reading_note_service: ReadingNoteService<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<ReadingNoteVisibleWithTags>, SearchJournalError> {
    let reading_notes = reading_note_service.search(user.id, text_query, tag_ids).await?;

    let mut res: Vec<ReadingNoteVisibleWithTags> = vec![];
    for reading_note in reading_notes {
        if res.is_empty() || res.last().unwrap().id != reading_note.id {
            let tags = match reading_note.tag_id {
                Some(_) => {
                    let reading_note_ = reading_note.clone();
                    vec![TagVisible {
                        id: reading_note_.tag_id.unwrap(),
                        name: reading_note_.tag_name.unwrap(),
                        r#type: reading_note_.tag_type.unwrap(),
                        created_at: reading_note_.tag_created_at.unwrap(),
                    }]
                }
                None => vec![],
            };
            let res_reading_note = ReadingNoteVisibleWithTags::from((reading_note, tags));
            res.push(res_reading_note);
        } else {
            if let Some(_) = reading_note.tag_id {
                res.last_mut().unwrap().push_tag(TagVisible {
                    id: reading_note.tag_id.unwrap(),
                    name: reading_note.tag_name.unwrap(),
                    r#type: reading_note.tag_type.unwrap(),
                    created_at: reading_note.tag_created_at.unwrap(),
                });
            }
        }
    }
    Ok(res)
}

async fn get_thinking_notes(
    thinking_note_service: ThinkingNoteService<'_>,
    user: &user_entity::Model,
    text_query: Vec<String>,
    tag_ids: Vec<Uuid>,
) -> Result<Vec<ThinkingNoteVisibleWithTags>, SearchJournalError> {
    let thinking_notes = thinking_note_service.search(user.id, text_query, tag_ids).await?;

    let mut res: Vec<ThinkingNoteVisibleWithTags> = vec![];
    for thinking_note in thinking_notes {
        if res.is_empty() || res.last().unwrap().id != thinking_note.id {
            let tags = match thinking_note.tag_id {
                Some(_) => {
                    let thinking_note_ = thinking_note.clone();
                    vec![TagVisible {
                        id: thinking_note_.tag_id.unwrap(),
                        name: thinking_note_.tag_name.unwrap(),
                        r#type: thinking_note_.tag_type.unwrap(),
                        created_at: thinking_note_.tag_created_at.unwrap(),
                    }]
                }
                None => vec![],
            };
            let res_thinking_note = ThinkingNoteVisibleWithTags::from((thinking_note, tags));
            res.push(res_thinking_note);
        } else {
            if let Some(_) = thinking_note.tag_id {
                res.last_mut().unwrap().push_tag(TagVisible {
                    id: thinking_note.tag_id.unwrap(),
                    name: thinking_note.tag_name.unwrap(),
                    r#type: thinking_note.tag_type.unwrap(),
                    created_at: thinking_note.tag_created_at.unwrap(),
                });
            }
        }
    }
    Ok(res)
}
