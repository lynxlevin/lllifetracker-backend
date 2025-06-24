use uuid::Uuid;

use crate::UseCaseError;
use db_adapters::action_track_adapter::{
    ActionTrackAdapter, ActionTrackFilter, ActionTrackMutation, ActionTrackQuery,
};
use entities::user as user_entity;

pub async fn delete_action_track<'a>(
    user: user_entity::Model,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
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

    action_track_adapter
        .delete(action_track)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
