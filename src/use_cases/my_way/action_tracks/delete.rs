use uuid::Uuid;

use crate::UseCaseError;
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackLimit, ActionTrackMutation,
        ActionTrackOrder, ActionTrackQuery,
    },
    user_adapter::{UserAdapter, UserMutation},
    Order::Asc,
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

    let original_started_at = action_track.started_at.clone();

    action_track_adapter
        .clone()
        .delete(action_track)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if user
        .first_track_at
        .is_some_and(|timestamp| timestamp == original_started_at)
    {
        let action_tracks = action_track_adapter
            .filter_eq_user(&user)
            .filter_eq_archived_action(false)
            .order_by_started_at(Asc)
            .limit(1)
            .get_all()
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
        user_adapter
            .update_first_track_at(
                user,
                match action_tracks.len() > 0 {
                    true => Some(action_tracks[0].started_at),
                    false => None,
                },
            )
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    }

    Ok(())
}
