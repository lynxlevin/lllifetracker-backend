use db_adapters::reading_note_adapter::{
    ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteMutation, ReadingNoteQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_reading_note<'a>(
    user: user_entity::Model,
    reading_note_id: Uuid,
    reading_note_adapter: ReadingNoteAdapter<'a>,
) -> Result<(), UseCaseError> {
    let reading_note = match reading_note_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(reading_note_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(reading_note) => reading_note,
        None => return Ok(()),
    };

    reading_note_adapter
        .delete(reading_note)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
