use domain_services::action_track_service::{
    ActionTrackDeleteParams, ActionTrackService, ActionTrackServiceError, ActionTrackServiceMutation,
};
use thiserror::Error;
use uuid::Uuid;

use crate::users::first_track_at_synchronizer::FirstTrackAtSynchronizer;
use db_adapters::{action_track_adapter::ActionTrackAdapter, user_adapter::UserAdapter};
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum DeleteActionTracksError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for DeleteActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        match e {
            ActionTrackServiceError::ActionTrackNotFound() => Self::NotFound(e.to_string()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        action_track_id = action_track_id.to_string(),
    ),
    skip_all
)]
pub async fn delete_action_track<'a>(
    user: user_entity::Model,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
    action_track_service: ActionTrackService<'a>,
) -> Result<(), DeleteActionTracksError> {
    let deleted_action_track = action_track_service
        .delete(ActionTrackDeleteParams { user: &user, action_track_id })
        .await?;

    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(Some(deleted_action_track), None)
        .await
        .map_err(|e| DeleteActionTracksError::InternalServerError(e.to_string()))?;

    Ok(())
}
