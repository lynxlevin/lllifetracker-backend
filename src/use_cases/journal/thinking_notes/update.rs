use db_adapters::{
    thinking_note_adapter::{
        ThinkingNoteAdapter, ThinkingNoteFilter, ThinkingNoteJoin, ThinkingNoteMutation,
        ThinkingNoteQuery, UpdateThinkingNoteParams,
    },
    CustomDbErr,
};
use entities::{tag, thinking_note, user as user_entity};
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    journal::thinking_notes::types::{ThinkingNoteUpdateRequest, ThinkingNoteVisible},
    UseCaseError,
};

pub async fn update_thinking_note<'a>(
    user: user_entity::Model,
    params: ThinkingNoteUpdateRequest,
    thinking_note_id: Uuid,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<ThinkingNoteVisible, UseCaseError> {
    let (thinking_note, linked_tags) = thinking_note_adapter
        .clone()
        .join_tags()
        .filter_eq_id(thinking_note_id)
        .filter_eq_user(&user)
        .get_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Thinking note with this id was not found".to_string(),
        ))?;

    let thinking_note = thinking_note_adapter
        .clone()
        .update(
            UpdateThinkingNoteParams {
                question: params.question,
                thought: params.thought,
                answer: params.answer,
                resolved_at: params.resolved_at,
                archived_at: params.archived_at,
            },
            thinking_note,
        )
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if let Err(e) = _update_tag_links(
        &thinking_note,
        linked_tags,
        params.tag_ids,
        thinking_note_adapter,
    )
    .await
    {
        match &e {
            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                CustomDbErr::NotFound => {
                    return Err(UseCaseError::NotFound(
                        "One or more of the tag_ids do not exist.".to_string(),
                    ))
                }
                _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            },
            // FIXME: thinking_note creation should be canceled.
            _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        }
    }

    Ok(ThinkingNoteVisible::from(thinking_note))
}

async fn _update_tag_links(
    thinking_note: &thinking_note::Model,
    linked_tags: Vec<tag::Model>,
    tag_ids: Vec<Uuid>,
    thinking_note_adapter: ThinkingNoteAdapter<'_>,
) -> Result<(), DbErr> {
    let linked_tag_ids = linked_tags.iter().map(|tag| tag.id).collect::<Vec<_>>();

    let tag_ids_to_link = tag_ids
        .clone()
        .into_iter()
        .filter(|id| !linked_tag_ids.contains(id));
    thinking_note_adapter
        .link_tags(&thinking_note, tag_ids_to_link)
        .await?;

    let tag_ids_to_unlink = linked_tag_ids
        .into_iter()
        .filter(|id| !tag_ids.contains(id));
    thinking_note_adapter
        .unlink_tags(&thinking_note, tag_ids_to_unlink)
        .await?;

    Ok(())
}
