use db_adapters::{
    direction_adapter::{CreateDirectionParams, DirectionAdapter, DirectionMutation},
    direction_category_adapter::{
        DirectionCategoryAdapter, DirectionCategoryFilter, DirectionCategoryQuery,
    },
};
use entities::user as user_entity;

use crate::{
    my_way::directions::types::{DirectionCreateRequest, DirectionVisible},
    UseCaseError,
};

pub async fn create_direction<'a>(
    user: user_entity::Model,
    params: DirectionCreateRequest,
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
    direction_adapter
        .create_with_tag(CreateDirectionParams {
            name: params.name.clone(),
            description: params.description.clone(),
            category_id,
            user_id: user.id,
        })
        .await
        .map(|direction| DirectionVisible::from(direction))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
