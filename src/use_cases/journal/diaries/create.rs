use db_adapters::{
    diary_adapter::{CreateDiaryParams, DiaryAdapter, DiaryMutation},
    CustomDbErr,
};
use entities::user as user_entity;
use sea_orm::DbErr;

use crate::{
    journal::diaries::types::{DiaryCreateRequest, DiaryVisible},
    UseCaseError,
};

pub async fn create_diary<'a>(
    user: user_entity::Model,
    params: DiaryCreateRequest,
    diary_adapter: DiaryAdapter<'a>,
) -> Result<DiaryVisible, UseCaseError> {
    let diary = match diary_adapter
        .clone()
        .create(CreateDiaryParams {
            text: params.text.clone(),
            date: params.date,
            user_id: user.id,
        })
        .await
    {
        Ok(diary) => diary,
        Err(e) => match &e {
            _ => return Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    };

    match diary_adapter
        .link_tags(&diary, params.tag_ids.clone())
        .await
    {
        Ok(_) => Ok(DiaryVisible::from(diary)),
        Err(e) => match &e {
            // FIXME: diary creation should be canceled.
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
