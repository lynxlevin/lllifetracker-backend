use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{my_way::desired_states::types::DesiredStateVisible, UseCaseError};

pub async fn unarchive_desired_state<'a>(
    user: user_entity::Model,
    desired_state_id: Uuid,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<DesiredStateVisible, UseCaseError> {
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
        .unarchive(desired_state)
        .await
        .map(|desired_state| DesiredStateVisible::from(desired_state))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
