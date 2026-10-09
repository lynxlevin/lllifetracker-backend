use domain_services::action_track_service::{
    ActionTrackService, ActionTrackServiceError, ActionTrackServiceMutation, ActionTrackUpdateParams,
};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    my_way::action_tracks::types::{ActionTrackUpdateRequest, ActionTrackVisible},
    users::first_track_at_synchronizer::FirstTrackAtSynchronizer,
};
use db_adapters::{action_track_adapter::ActionTrackAdapter, user_adapter::UserAdapter};
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum UpdateActionTracksError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for UpdateActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        match e {
            ActionTrackServiceError::DuplicateCreation() => Self::ValidationError(e.to_string()),
            ActionTrackServiceError::ActionTrackNotFound() => Self::NotFound(e.to_string()),
            ActionTrackServiceError::ActionNotFound() => Self::NotFound(e.to_string()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        action_track_id = action_track_id.to_string(),
        params.action_id = params.action_id.to_string(),
        params.started_at.is_some = params.started_at.to_string(),
        params.ended_at.is_some = params.ended_at.is_some(),
    ),
    skip_all
)]
pub async fn update_action_track<'a>(
    user: user_entity::Model,
    params: ActionTrackUpdateRequest,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
    action_track_service: ActionTrackService<'a>,
) -> Result<ActionTrackVisible, UpdateActionTracksError> {
    let params = _parse_params(params)?;

    let (new_action_track, old_action_track) = action_track_service
        .update(ActionTrackUpdateParams {
            user: &user,
            action_track_id: action_track_id,
            action_id: params.action_id,
            started_at: params.started_at,
            ended_at: params.ended_at,
        })
        .await?;

    // MYMEMO: Incorporate this into the service.
    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(Some(old_action_track), Some(new_action_track.clone()))
        .await
        .map_err(|e| UpdateActionTracksError::InternalServerError(e.to_string()))?;

    Ok(ActionTrackVisible::from(new_action_track))
}

fn _parse_params(params: ActionTrackUpdateRequest) -> Result<ActionTrackUpdateRequest, UpdateActionTracksError> {
    if params.ended_at.is_some_and(|ended_at| ended_at < params.started_at) {
        return Err(UpdateActionTracksError::ValidationError(
            "Ended_at must be later than started_at.".to_string(),
        ));
    }

    Ok(params)
}
