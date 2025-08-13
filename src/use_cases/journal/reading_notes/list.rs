use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteJoin, ReadingNoteOrder, ReadingNoteQuery,
        ReadingNoteWithTag,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;

use crate::{
    journal::reading_notes::types::ReadingNoteVisibleWithTags,
    tags::types::{TagType, TagVisible},
    UseCaseError,
};

pub async fn list_reading_notes<'a>(
    user: user_entity::Model,
    reading_note_adapter: ReadingNoteAdapter<'a>,
) -> Result<Vec<ReadingNoteVisibleWithTags>, UseCaseError> {
    let reading_notes = reading_note_adapter
        .filter_eq_user(&user)
        .join_tags()
        .join_my_way_via_tags()
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
        if res.is_empty() || res.last().unwrap().id != reading_note.id {
            let mut res_reading_note = ReadingNoteVisibleWithTags {
                id: reading_note.id,
                title: reading_note.title.clone(),
                page_number: reading_note.page_number,
                text: reading_note.text.clone(),
                date: reading_note.date,
                created_at: reading_note.created_at,
                updated_at: reading_note.updated_at,
                tags: vec![],
            };
            if let Some(tag) = get_tag(&reading_note) {
                res_reading_note.push_tag(tag);
            }
            res.push(res_reading_note);
        } else {
            if let Some(tag) = get_tag(&reading_note) {
                res.last_mut().unwrap().push_tag(tag);
            }
        }
    }
    Ok(res)
}

fn get_tag(reading_note: &ReadingNoteWithTag) -> Option<TagVisible> {
    if reading_note.tag_id.is_none() {
        return None;
    }

    let (name, tag_type) = if let Some(name) = reading_note.tag_name.clone() {
        (name, TagType::Plain)
    } else if let Some(name) = reading_note.tag_ambition_name.clone() {
        (name, TagType::Ambition)
    } else if let Some(name) = reading_note.tag_desired_state_name.clone() {
        (name, TagType::DesiredState)
    } else if let Some(name) = reading_note.tag_action_name.clone() {
        (name, TagType::Action)
    } else {
        panic!("Tag without name should not exist.");
    };

    Some(TagVisible {
        id: reading_note.tag_id.unwrap(),
        name,
        tag_type,
        created_at: reading_note.tag_created_at.unwrap(),
    })
}
