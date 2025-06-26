use db_adapters::diary_adapter::{DiaryAdapter, DiaryFilter, DiaryMutation, DiaryQuery};
use entities::user as user_entity;
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_diary<'a>(
    user: user_entity::Model,
    diary_id: Uuid,
    diary_adapter: DiaryAdapter<'a>,
) -> Result<(), UseCaseError> {
    let diary = match diary_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(diary_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(diary) => diary,
        None => return Ok(()),
    };
    diary_adapter
        .delete(diary)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
