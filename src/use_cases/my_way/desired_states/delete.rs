use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
};
use entities::user as user_entity;
use uuid::Uuid;

use crate::UseCaseError;

pub async fn delete_desired_state<'a>(
    user: user_entity::Model,
    desired_state_id: Uuid,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<(), UseCaseError> {
    let desired_state = match desired_state_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(desired_state_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(desired_state) => desired_state,
        None => return Ok(()),
    };
    desired_state_adapter
        .delete(desired_state)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
