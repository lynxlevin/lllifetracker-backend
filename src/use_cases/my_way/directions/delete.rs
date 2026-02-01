use db_adapters::direction_adapter::{
    DirectionAdapter, DirectionFilter, DirectionMutation, DirectionQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_direction<'a>(
    user: user_entity::Model,
    direction_id: Uuid,
    direction_adapter: DirectionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let direction = match direction_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(direction_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(direction) => direction,
        None => return Ok(()),
    };
    direction_adapter
        .delete(direction)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
