use uuid::Uuid;

use crate::{my_way::actions::types::ActionVisible, UseCaseError};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;

pub async fn unarchive_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
) -> Result<ActionVisible, UseCaseError> {
    let action = action_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Action with this id was not found".to_string(),
        ))?;

    action_adapter
        .unarchive(action)
        .await
        .map(|action| ActionVisible::from(action))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
