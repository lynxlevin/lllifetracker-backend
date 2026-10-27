use domain_services::action_service::{
    ActionConvertTrackTypeParams, ActionService, ActionServiceError, ActionServiceMutation,
};
use thiserror::Error;
use uuid::Uuid;

use crate::my_way::actions::types::{ActionTrackTypeConversionRequest, ActionVisible};
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum ConvertTrackTypeActionError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for ConvertTrackTypeActionError {
    fn from(e: ActionServiceError) -> Self {
        match e {
            ActionServiceError::ActionNotFound() => Self::NotFound(e.to_string()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        action_id = action_id.to_string(),
        params.track_type = params.track_type.to_string(),
    ),
    skip_all
)]
pub async fn convert_action_track_type<'a>(
    user: user_entity::Model,
    params: ActionTrackTypeConversionRequest,
    action_id: Uuid,
    action_service: ActionService<'a>,
) -> Result<ActionVisible, ConvertTrackTypeActionError> {
    let action = action_service
        .convert_track_type(ActionConvertTrackTypeParams { user, action_id, track_type: params.track_type })
        .await?;

    Ok(ActionVisible::from(action))
}
