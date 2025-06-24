use crate::{
    my_way::actions::types::{ActionCreateRequest, ActionVisible},
    UseCaseError,
};
use db_adapters::action_adapter::{ActionAdapter, ActionMutation, CreateActionParams};
use entities::user as user_entity;

pub async fn create_action<'a>(
    user: user_entity::Model,
    params: ActionCreateRequest,
    action_adapter: ActionAdapter<'a>,
) -> Result<ActionVisible, UseCaseError> {
    action_adapter
        .create_with_tag(CreateActionParams {
            name: params.name.clone(),
            description: params.description.clone(),
            track_type: params.track_type.clone(),
            user_id: user.id,
        })
        .await
        .map(|action| ActionVisible::from(action))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
