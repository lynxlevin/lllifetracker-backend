use db_adapters::{
    desired_state_adapter::{CreateDesiredStateParams, DesiredStateAdapter, DesiredStateMutation},
    desired_state_category_adapter::{
        DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryQuery,
    },
};
use entities::user as user_entity;

use crate::{
    my_way::desired_states::types::{DesiredStateCreateRequest, DesiredStateVisible},
    UseCaseError,
};

pub async fn create_desired_state<'a>(
    user: user_entity::Model,
    params: DesiredStateCreateRequest,
    desired_state_adapter: DesiredStateAdapter<'a>,
    category_adapter: DesiredStateCategoryAdapter<'a>,
) -> Result<DesiredStateVisible, UseCaseError> {
    let category_id = match params.category_id {
        Some(category_id) => category_adapter
            .filter_eq_user(&user)
            .get_by_id(category_id)
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
            .and(Some(category_id)),
        None => None,
    };
    desired_state_adapter
        .create_with_tag(CreateDesiredStateParams {
            name: params.name.clone(),
            description: params.description.clone(),
            category_id,
            user_id: user.id,
        })
        .await
        .map(|desired_state| DesiredStateVisible::from(desired_state))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
