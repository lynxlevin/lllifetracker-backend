use uuid::Uuid;

use crate::{
    my_way::direction_categories::types::{DirectionCategoryUpdateRequest, DirectionCategoryVisible},
    UseCaseError,
};
use db_adapters::direction_category_adapter::{
    DirectionCategoryAdapter, DirectionCategoryFilter, DirectionCategoryMutation, DirectionCategoryQuery,
    UpdateDirectionCategoryParams,
};
use entities::user as user_entity;

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        category_id = category_id.to_string(),
        params.name.len = params.name.len(),
    ),
    skip_all
)]
pub async fn update_direction_category<'a>(
    user: user_entity::Model,
    params: DirectionCategoryUpdateRequest,
    category_id: Uuid,
    category_adapter: DirectionCategoryAdapter<'a>,
) -> Result<DirectionCategoryVisible, UseCaseError> {
    let category = category_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(category_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound("Category not found".to_string()))?;

    category_adapter
        .update(category, UpdateDirectionCategoryParams { name: params.name.clone() })
        .await
        .map(|category| DirectionCategoryVisible::from(category))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
