use uuid::Uuid;

use crate::{my_way::actions::types::ActionVisible, UseCaseError};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery},
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackLimit, ActionTrackOrder, ActionTrackQuery,
    },
    user_adapter::{UserAdapter, UserMutation},
    Order::Asc,
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

    if user.first_track_at.is_some() {
        let action_tracks = action_track_adapter
            .filter_eq_user(&user)
            .filter_eq_archived_action(false)
            .order_by_started_at(Asc)
            .limit(1)
            .get_all()
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

        if action_tracks.len() > 0 {
            if user.first_track_at.unwrap() != action_tracks[0].started_at {
                user_adapter
                    .update_first_track_at(user, Some(action_tracks[0].started_at))
                    .await
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
            }
        } else {
            user_adapter
                .update_first_track_at(user, None)
                .await
                .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
        };
    }

    Ok(ActionVisible::from(action))
}
