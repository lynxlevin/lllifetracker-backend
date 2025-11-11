use db_adapters::{
    thinking_note_adapter::{
        ThinkingNoteAdapter, ThinkingNoteFilter, ThinkingNoteJoin, ThinkingNoteOrder,
        ThinkingNoteQuery, ThinkingNoteWithTag,
    },
    Order::{Asc, Desc},
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{
    journal::{
        thinking_notes::types::{ThinkingNoteListQuery, ThinkingNoteVisibleWithTags},
        types::IntoJournalVisibleWithTags,
    },
    tags::types::TagVisible,
    UseCaseError,
};

pub async fn list_thinking_notes<'a>(
    user: user_entity::Model,
    params: ThinkingNoteListQuery,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<Vec<ThinkingNoteVisibleWithTags>, UseCaseError> {
    let params = validate_params(params)?;
    let mut query = thinking_note_adapter
        .join_tags()
        .join_my_way_via_tags()
        .filter_eq_user(&user);

    if let Some(resolved) = params.resolved {
        query = query.filter_null_resolved_at(!resolved);
    }

    let thinking_notes = query
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

    // NOTE: This filtering cannot be done in Db query.
    // If done in DB query, tags not in tag_id_or will be returned.
    if let Some(tag_id_or) = params.tag_id_or {
        res = res
            .into_iter()
            .filter(|thinking_note| {
                thinking_note
                    .tags
                    .iter()
                    .find(|tag| tag_id_or.contains(&tag.id))
                    .is_some()
            })
            .collect();
    }

    Ok(res)
}

struct QueryParam {
    resolved: Option<bool>,
    tag_id_or: Option<Vec<Uuid>>,
}

fn validate_params(params: ThinkingNoteListQuery) -> Result<QueryParam, UseCaseError> {
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

    Ok(QueryParam {
        tag_id_or,
        resolved: params.resolved,
    })
}

fn first_to_process(
    res: &Vec<ThinkingNoteVisibleWithTags>,
    thinking_note: &ThinkingNoteWithTag,
) -> bool {
    res.is_empty() || res.last().unwrap().id != thinking_note.id
}
