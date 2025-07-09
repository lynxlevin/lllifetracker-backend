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

    let should_update_user_first_track_at =
        user.first_track_at.is_some() && user.first_track_at.unwrap() == action_track.started_at;

    action_track_adapter
        .clone()
        .delete(action_track)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if should_update_user_first_track_at {
        let action_tracks = action_track_adapter
            .filter_eq_user(&user)
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
