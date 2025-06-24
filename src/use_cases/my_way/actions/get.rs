use uuid::Uuid;

use crate::{my_way::actions::types::ActionVisible, UseCaseError};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionQuery};
use entities::user as user_entity;

pub async fn get_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
) -> Result<ActionVisible, UseCaseError> {
    action_adapter
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .map(|action| ActionVisible::from(action))
        .ok_or(UseCaseError::NotFound(
            "Action with this id was not found".to_string(),
        ))
}
