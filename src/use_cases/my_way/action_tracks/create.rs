use domain_services::action_track_service::{
    ActionTrackCreateParams, ActionTrackService, ActionTrackServiceError, ActionTrackServiceMutation,
};
use thiserror::Error;

use crate::{
    my_way::action_tracks::types::{ActionTrackCreateRequest, ActionTrackVisible},
    users::first_track_at_synchronizer::FirstTrackAtSynchronizer,
};
use db_adapters::{action_track_adapter::ActionTrackAdapter, user_adapter::UserAdapter};
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum CreateActionTracksError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for CreateActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        match e {
            ActionTrackServiceError::DuplicateCreation() => Self::ValidationError(e.to_string()),
            ActionTrackServiceError::ActionNotFound() => Self::NotFound(e.to_string()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        req.started_at = req.started_at.to_string(),
        req.action_id = req.action_id.to_string(),
    ),
    skip_all
)]
pub async fn create_action_track<'a>(
    user: user_entity::Model,
    req: ActionTrackCreateRequest,
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
    action_track_service: ActionTrackService<'a>,
) -> Result<ActionTrackVisible, CreateActionTracksError> {
    let action_track = action_track_service
        .create(ActionTrackCreateParams { user: &user, action_id: req.action_id, started_at: req.started_at })
        .await?;

    // MYMEMO: Incorporate this into the service.
    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(None, Some(action_track.clone()))
        .await
        .map_err(|e| CreateActionTracksError::InternalServerError(e.to_string()))?;

    Ok(ActionTrackVisible::from(action_track))
}
