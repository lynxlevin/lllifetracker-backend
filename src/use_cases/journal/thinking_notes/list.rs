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
    tags::types::TagVisible,
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
        .order_by_resolved_at_nulls_first(Desc)
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
        if first_to_process(&res, &thinking_note) {
            let tags = match thinking_note.tag_id {
                Some(_) => vec![Into::<TagVisible>::into(&thinking_note)],
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

fn first_to_process(
    res: &Vec<ThinkingNoteVisibleWithTags>,
    thinking_note: &ThinkingNoteWithTag,
) -> bool {
    res.is_empty() || res.last().unwrap().id != thinking_note.id
}
