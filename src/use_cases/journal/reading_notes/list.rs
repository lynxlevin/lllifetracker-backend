use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteJoin, ReadingNoteOrder, ReadingNoteQuery,
        ReadingNoteWithTag,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{
    journal::{
        reading_notes::types::{ReadingNoteListQuery, ReadingNoteVisibleWithTags},
        types::IntoJournalVisibleWithTags,
    },
    tags::types::TagVisible,
    UseCaseError,
};

pub async fn list_reading_notes<'a>(
    user: user_entity::Model,
    reading_note_adapter: ReadingNoteAdapter<'a>,
    params: ReadingNoteListQuery,
) -> Result<Vec<ReadingNoteVisibleWithTags>, UseCaseError> {
    let params = validate_params(params)?;
    let mut query = reading_note_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(&user);

    if let Some(tag_id_or) = params.tag_id_or {
        query = query.filter_in_tag_ids_or(tag_id_or);
    }

    let reading_notes = query
        .order_by_date(Desc)
        .order_by_created_at(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_desired_state_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ReadingNoteVisibleWithTags> = vec![];
    for reading_note in reading_notes {
        if first_to_process(&res, &reading_note) {
            let tags = match reading_note.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&reading_note)],
                None => vec![],
            };
            let res_reading_note = ReadingNoteVisibleWithTags {
                id: reading_note.id,
                title: reading_note.title,
                page_number: reading_note.page_number,
                text: reading_note.text,
                date: reading_note.date,
                created_at: reading_note.created_at,
                updated_at: reading_note.updated_at,
                tags,
            };
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

struct QueryParam {
    tag_id_or: Option<Vec<Uuid>>,
}

fn validate_params(params: ReadingNoteListQuery) -> Result<QueryParam, UseCaseError> {
    let tag_id_or: Option<Vec<Uuid>> = params.tag_id_or.and_then(|tag_id_or| {
        Some(
            tag_id_or
                .split(',')
                .map(|tag_id| {
                    Uuid::parse_str(tag_id)
                        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
                })
                .filter(|tag_id| tag_id.is_ok())
                .map(|tag_id| tag_id.unwrap())
                .collect(),
        )
    });

    Ok(QueryParam { tag_id_or })
}

fn first_to_process(
    res: &Vec<ReadingNoteVisibleWithTags>,
    reading_note: &ReadingNoteWithTag,
) -> bool {
    res.is_empty() || res.last().unwrap().id != reading_note.id
}
