use crate::my_way::action_tracks::types::{ActionTrackListQuery, ActionTrackVisible};
use domain_services::action_track_service::{
    ActionTrackListParams, ActionTrackService, ActionTrackServiceError, ActionTrackServiceQuery,
};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListActionTracksError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionTrackServiceError> for ListActionTracksError {
    fn from(e: ActionTrackServiceError) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.active_only = params.active_only,
        params.started_at_gte.is_some = params.started_at_gte.is_some(),
        params.started_at_lte.is_some = params.started_at_lte.is_some(),
    ),
    skip_all
)]
pub async fn list_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackListQuery,
    action_track_service: ActionTrackService<'a>,
) -> Result<Vec<ActionTrackVisible>, ListActionTracksError> {
    let action_tracks = action_track_service
        .list(
            user.id,
            ActionTrackListParams {
                active_only: params.active_only,
                started_at_gte: params.started_at_gte,
                started_at_lte: params.started_at_lte,
            },
        )
        .await?;

    Ok(action_tracks
        .iter()
        .map(|at| ActionTrackVisible::from(at))
        .collect::<Vec<_>>())
}
