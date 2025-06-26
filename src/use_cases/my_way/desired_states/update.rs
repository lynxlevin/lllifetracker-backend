use db_adapters::{
    desired_state_adapter::{
        DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
        UpdateDesiredStateParams,
    },
    desired_state_category_adapter::{
        DesiredStateCategoryAdapter, DesiredStateCategoryFilter, DesiredStateCategoryQuery,
    },
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{
    my_way::desired_states::types::{DesiredStateUpdateRequest, DesiredStateVisible},
    UseCaseError,
};

pub async fn update_desired_state<'a>(
    user: user_entity::Model,
    params: DesiredStateUpdateRequest,
    desired_state_id: Uuid,
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

    let desired_state = desired_state_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(desired_state_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "DesiredState with this id was not found".to_string(),
        ))?;

    desired_state_adapter
        .update(
            desired_state,
            UpdateDesiredStateParams {
                name: params.name.clone(),
                description: params.description.clone(),
                category_id,
                is_focused: params.is_focused,
            },
        )
        .await
        .map(|desired_state| DesiredStateVisible::from(desired_state))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
