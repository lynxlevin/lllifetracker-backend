use db_adapters::direction_adapter::{DirectionAdapter, DirectionFilter, DirectionQuery};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{my_way::directions::types::DirectionVisible, UseCaseError};

pub async fn get_direction<'a>(
    user: user_entity::Model,
    direction_id: Uuid,
    direction_adapter: DirectionAdapter<'a>,
) -> Result<DirectionVisible, UseCaseError> {
    direction_adapter
        .filter_eq_user(&user)
        .get_by_id(direction_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Direction with this id was not found".to_string(),
        ))
        .map(|direction| DirectionVisible::from(direction))
}
