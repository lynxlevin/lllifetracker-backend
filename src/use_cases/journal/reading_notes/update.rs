use db_adapters::{
    reading_note_adapter::{
        ReadingNoteAdapter, ReadingNoteFilter, ReadingNoteJoin, ReadingNoteMutation,
        ReadingNoteQuery, UpdateReadingNoteParams,
    },
    CustomDbErr,
};
use entities::{reading_note, tag, user as user_entity};
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    journal::reading_notes::types::{ReadingNoteUpdateRequest, ReadingNoteVisible},
    UseCaseError,
};

pub async fn update_reading_note<'a>(
    user: user_entity::Model,
    params: ReadingNoteUpdateRequest,
    reading_note_id: Uuid,
    reading_note_adapter: ReadingNoteAdapter<'a>,
) -> Result<ReadingNoteVisible, UseCaseError> {
    let (reading_note, linked_tags) = reading_note_adapter
        .clone()
        .join_my_way_tags()
        .filter_eq_id(reading_note_id)
        .filter_eq_user(&user)
        .get_with_tags()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Reading note with this id was not found".to_string(),
        ))?;

    let reading_note = reading_note_adapter
        .clone()
        .partial_update(
            reading_note,
            UpdateReadingNoteParams {
                title: params.title.clone(),
                page_number: params.page_number,
                text: params.text.clone(),
                date: params.date,
            },
        )
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if let Some(tag_ids) = params.tag_ids.clone() {
        if let Err(e) =
            _update_tag_links(&reading_note, linked_tags, tag_ids, reading_note_adapter).await
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
                // FIXME: reading_note creation should be canceled.
                _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            }
        }
    }

    Ok(ReadingNoteVisible::from(reading_note))
}

async fn _update_tag_links(
    reading_note: &reading_note::Model,
    linked_tags: Vec<tag::Model>,
    tag_ids: Vec<Uuid>,
    reading_note_adapter: ReadingNoteAdapter<'_>,
) -> Result<(), DbErr> {
    let linked_tag_ids = linked_tags.iter().map(|tag| tag.id).collect::<Vec<_>>();

    let tag_ids_to_link = tag_ids
        .clone()
        .into_iter()
        .filter(|id| !linked_tag_ids.contains(id));
    reading_note_adapter
        .link_tags(&reading_note, tag_ids_to_link)
        .await?;

    let tag_ids_to_unlink = linked_tag_ids
        .into_iter()
        .filter(|id| !tag_ids.contains(id));
    reading_note_adapter
        .unlink_tags(&reading_note, tag_ids_to_unlink)
        .await?;

    Ok(())
}
