use uuid::Uuid;

use crate::UseCaseError;
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;

pub async fn delete_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let action = match action_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(action) => action,
        None => return Ok(()),
    };

    action_adapter
        .delete(action)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
