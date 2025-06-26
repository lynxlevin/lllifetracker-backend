use uuid::Uuid;

use crate::{
    my_way::desired_state_categories::types::{
        DesiredStateCategoryUpdateRequest, DesiredStateCategoryVisible,
    },
    UseCaseError,
};
use db_adapters::desired_state_category_adapter::{
    DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryMutation,
    DesiredStateCategoryQuery, UpdateDesiredStateCategoryParams,
};
use entities::user as user_entity;

pub async fn update_desired_state_category<'a>(
    user: user_entity::Model,
    params: DesiredStateCategoryUpdateRequest,
    category_id: Uuid,
    category_adapter: DesiredStateCategoryAdapter<'a>,
) -> Result<DesiredStateCategoryVisible, UseCaseError> {
    let category = category_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(category_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound("Category not found".to_string()))?;

    category_adapter
        .update(
            category,
            UpdateDesiredStateCategoryParams {
                name: params.name.clone(),
            },
        )
        .await
        .map(|category| DesiredStateCategoryVisible::from(category))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
