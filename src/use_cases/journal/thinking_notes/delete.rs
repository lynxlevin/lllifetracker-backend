use db_adapters::thinking_note_adapter::{
    ThinkingNoteAdapter, ThinkingNoteFilter, ThinkingNoteMutation, ThinkingNoteQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_thinking_note<'a>(
    user: user_entity::Model,
    thinking_note_id: Uuid,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<(), UseCaseError> {
    let thinking_note = match thinking_note_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(thinking_note_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(thinking_note) => thinking_note,
        None => return Ok(()),
    };

    thinking_note_adapter
        .delete(thinking_note)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
