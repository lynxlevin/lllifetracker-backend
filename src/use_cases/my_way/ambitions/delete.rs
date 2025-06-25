use uuid::Uuid;

use crate::UseCaseError;
use db_adapters::ambition_adapter::{
    AmbitionAdapter, AmbitionFilter, AmbitionMutation, AmbitionQuery,
};
use entities::user as user_entity;

pub async fn delete_ambition<'a>(
    user: user_entity::Model,
    ambition_id: Uuid,
    ambition_adapter: AmbitionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let ambition = match ambition_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(ambition_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(ambition) => ambition,
        None => return Ok(()),
    };
    ambition_adapter
        .delete(ambition)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
