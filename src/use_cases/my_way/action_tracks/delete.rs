use uuid::Uuid;

use crate::{users::first_track_at_synchronizer::FirstTrackAtSynchronizer, UseCaseError};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackMutation, ActionTrackQuery,
    },
    user_adapter::UserAdapter,
};
use entities::user as user_entity;

pub async fn delete_action_track<'a>(
    user: user_entity::Model,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
) -> Result<(), UseCaseError> {
    let action_track = match action_track_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_track_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(action_track) => action_track,
        None => return Ok(()),
    };

    let old_action_track = action_track.clone();

    action_track_adapter
        .clone()
        .delete(action_track)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(Some(old_action_track), None)
        .await?;

    Ok(())
}
