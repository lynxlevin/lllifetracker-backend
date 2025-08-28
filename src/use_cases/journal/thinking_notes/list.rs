use db_adapters::{
    thinking_note_adapter::{
        ThinkingNoteAdapter, ThinkingNoteFilter, ThinkingNoteJoin, ThinkingNoteOrder,
        ThinkingNoteQuery, ThinkingNoteWithTag,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;

use crate::{
    journal::thinking_notes::types::{ThinkingNoteListQuery, ThinkingNoteVisibleWithTags},
    tags::types::{TagType, TagVisible},
    UseCaseError,
};

pub async fn list_thinking_notes<'a>(
    user: user_entity::Model,
    params: ThinkingNoteListQuery,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<Vec<ThinkingNoteVisibleWithTags>, UseCaseError> {
    let thinking_notes = thinking_note_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(&user)
        .filter_null_resolved_at(!params.resolved.unwrap_or(false))
        .filter_null_archived_at(!params.archived.unwrap_or(false))
        .order_by_updated_at(Desc)
        .order_by_ambition_created_at_nulls_last(Asc)
        .order_by_desired_state_created_at_nulls_last(Asc)
        .order_by_action_created_at_nulls_last(Asc)
        .order_by_tag_created_at_nulls_last(Asc)
        .get_all_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ThinkingNoteVisibleWithTags> = vec![];
    for thinking_note in thinking_notes {
        let same_thinking_note_as_prev =
            !res.is_empty() && res.last().unwrap().id == thinking_note.id;

        if same_thinking_note_as_prev {
            if let Some(tag) = _get_tag(&thinking_note) {
                res.last_mut().unwrap().push_tag(tag);
            }
        } else {
            let tags = match _get_tag(&thinking_note) {
                Some(tag) => vec![tag],
                None => vec![],
            };
            let res_thinking_note = ThinkingNoteVisibleWithTags {
                id: thinking_note.id,
                question: thinking_note.question,
                thought: thinking_note.thought,
                answer: thinking_note.answer,
                resolved_at: thinking_note.resolved_at,
                archived_at: thinking_note.archived_at,
                created_at: thinking_note.created_at,
                updated_at: thinking_note.updated_at,
                tags,
            };
            res.push(res_thinking_note);
        }
    }

    Ok(res)
}

fn _get_tag(thinking_note: &ThinkingNoteWithTag) -> Option<TagVisible> {
    if thinking_note.tag_id.is_none() {
        return None;
    }

    let (name, tag_type) = if let Some(name) = thinking_note.tag_name.clone() {
        (name, TagType::Plain)
    } else if let Some(name) = thinking_note.tag_ambition_name.clone() {
        (name, TagType::Ambition)
    } else if let Some(name) = thinking_note.tag_desired_state_name.clone() {
        (name, TagType::DesiredState)
    } else if let Some(name) = thinking_note.tag_action_name.clone() {
        (name, TagType::Action)
    } else {
        panic!("Tag without name should not exist.");
    };

    Some(TagVisible {
        id: thinking_note.tag_id.unwrap(),
        name,
        tag_type,
        created_at: thinking_note.tag_created_at.unwrap(),
    })
}
