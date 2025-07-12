use uuid::Uuid;

use crate::{
    my_way::actions::types::ActionVisible,
    users::first_track_at_synchronizer::FirstTrackAtSynchronizer, UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery},
    action_track_adapter::ActionTrackAdapter,
    user_adapter::UserAdapter,
};
use entities::user as user_entity;

pub async fn archive_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
    user_adapter: UserAdapter<'a>,
    action_track_adapter: ActionTrackAdapter<'a>,
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

    let action = action_adapter
        .archive(action)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(None, None)
        .await?;

    Ok(ActionVisible::from(action))
}
