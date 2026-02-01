use db_adapters::{
    direction_adapter::{
        DirectionAdapter, DirectionFilter, DirectionMutation, DirectionQuery,
        UpdateDirectionParams,
    },
    direction_category_adapter::{
        DirectionCategoryAdapter, DirectionCategoryFilter, DirectionCategoryQuery,
    },
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{
    my_way::directions::types::{DirectionUpdateRequest, DirectionVisible},
    UseCaseError,
};

pub async fn update_direction<'a>(
    user: user_entity::Model,
    params: DirectionUpdateRequest,
    direction_id: Uuid,
    direction_adapter: DirectionAdapter<'a>,
    category_adapter: DirectionCategoryAdapter<'a>,
) -> Result<DirectionVisible, UseCaseError> {
    let category_id = match params.category_id {
        Some(category_id) => category_adapter
            .filter_eq_user(&user)
            .get_by_id(category_id)
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
            .and(Some(category_id)),
        None => None,
    };

    let direction = direction_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(direction_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Direction with this id was not found".to_string(),
        ))?;

    direction_adapter
        .update(
            direction,
            UpdateDirectionParams {
                name: params.name.clone(),
                description: params.description.clone(),
                category_id,
            },
        )
        .await
        .map(|direction| DirectionVisible::from(direction))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
