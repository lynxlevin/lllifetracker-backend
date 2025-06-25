use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::{my_way::desired_states::types::DesiredStateVisible, UseCaseError};

pub async fn get_desired_state<'a>(
    user: user_entity::Model,
    desired_state_id: Uuid,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<DesiredStateVisible, UseCaseError> {
    desired_state_adapter
        .filter_eq_user(&user)
        .get_by_id(desired_state_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "DesiredState with this id was not found".to_string(),
        ))
        .map(|desired_state| DesiredStateVisible::from(desired_state))
}
