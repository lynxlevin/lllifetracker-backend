use uuid::Uuid;

use db_adapters::direction_category_adapter::{
    DirectionCategoryAdapter, DirectionCategoryFilter, DirectionCategoryMutation,
    DirectionCategoryQuery,
};
use entities::user as user_entity;

use crate::UseCaseError;

pub async fn delete_direction_category<'a>(
    user: user_entity::Model,
    category_id: Uuid,
    category_adapter: DirectionCategoryAdapter<'a>,
) -> Result<(), UseCaseError> {
    let category = match category_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(category_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(category) => category,
        None => return Ok(()),
    };

    category_adapter
        .delete(category)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
