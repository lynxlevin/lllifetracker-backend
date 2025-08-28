use db_adapters::{
    thinking_note_adapter::{CreateThinkingNoteParams, ThinkingNoteAdapter, ThinkingNoteMutation},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::DbErr;

use crate::{
    journal::thinking_notes::types::{ThinkingNoteCreateRequest, ThinkingNoteVisible},
    UseCaseError,
};

pub async fn create_thinking_note<'a>(
    user: user_entity::Model,
    params: ThinkingNoteCreateRequest,
    thinking_note_adapter: ThinkingNoteAdapter<'a>,
) -> Result<ThinkingNoteVisible, UseCaseError> {
    let thinking_note = match thinking_note_adapter
        .clone()
        .create(CreateThinkingNoteParams {
            question: params.question,
            thought: params.thought,
            answer: params.answer,
            user_id: user.id,
        })
        .await
    {
        Ok(thinking_note) => thinking_note,
        Err(e) => match &e {
            _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    };

    match thinking_note_adapter
        .link_tags(&thinking_note, params.tag_ids.clone())
        .await
    {
        Ok(_) => Ok(ThinkingNoteVisible::from(thinking_note)),
        Err(e) => match &e {
            // FIXME: thinking_note creation should be canceled.
            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                CustomDbErr::NotFound => {
                    return Err(UseCaseError::NotFound(
                        "One or more of the tag_ids do not exist.".to_string(),
                    ))
                }
                _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            },
            _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    }
}
