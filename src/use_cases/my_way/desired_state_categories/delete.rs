use uuid::Uuid;

use db_adapters::desired_state_category_adapter::{
    DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryMutation,
    DesiredStateCategoryQuery,
};
use entities::user as user_entity;

use crate::UseCaseError;

pub async fn delete_desired_state_category<'a>(
    user: user_entity::Model,
    category_id: Uuid,
    category_adapter: DesiredStateCategoryAdapter<'a>,
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
