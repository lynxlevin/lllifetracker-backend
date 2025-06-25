use db_adapters::{
    reading_note_adapter::{CreateReadingNoteParams, ReadingNoteAdapter, ReadingNoteMutation},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::DbErr;

use crate::{
    journal::reading_notes::types::{ReadingNoteCreateRequest, ReadingNoteVisible},
    UseCaseError,
};

pub async fn create_reading_note<'a>(
    user: user_entity::Model,
    params: ReadingNoteCreateRequest,
    reading_note_adapter: ReadingNoteAdapter<'a>,
) -> Result<ReadingNoteVisible, UseCaseError> {
    let reading_note = reading_note_adapter
        .clone()
        .create(CreateReadingNoteParams {
            title: params.title.clone(),
            page_number: params.page_number,
            text: params.text.clone(),
            date: params.date,
            user_id: user.id,
        })
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    match reading_note_adapter
        .link_tags(&reading_note, params.tag_ids.clone())
        .await
    {
        Ok(_) => Ok(ReadingNoteVisible::from(reading_note)),
        // FIXME: reading_note creation should be canceled.
        Err(e) => match &e {
            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                CustomDbErr::NotFound => Err(UseCaseError::NotFound(
                    "One or more of the tag_ids do not exist.".to_string(),
                )),
                _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            },
            _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    }
}
