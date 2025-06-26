use crate::{
    my_way::desired_state_categories::types::{
        DesiredStateCategoryCreateRequest, DesiredStateCategoryVisible,
    },
    UseCaseError,
};
use db_adapters::desired_state_category_adapter::{
    CreateDesiredStateCategoryParams, DesiredStateCategoryAdapter, DesiredStateCategoryMutation,
};
use entities::user as user_entity;

pub async fn create_desired_state_category<'a>(
    user: user_entity::Model,
    params: DesiredStateCategoryCreateRequest,
    category_adapter: DesiredStateCategoryAdapter<'a>,
) -> Result<DesiredStateCategoryVisible, UseCaseError> {
    category_adapter
        .create(CreateDesiredStateCategoryParams {
            name: params.name.clone(),
            user_id: user.id,
        })
        .await
        .map(|category| DesiredStateCategoryVisible::from(category))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
