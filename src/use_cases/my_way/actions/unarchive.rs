use domain_services::action_service::{
    ActionService, ActionServiceError, ActionServiceMutation, ActionUnarchiveParams,
};
use thiserror::Error;
use uuid::Uuid;

use crate::my_way::actions::types::ActionVisible;
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum UnarchiveActionError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for UnarchiveActionError {
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
    ),
    skip_all
)]
pub async fn unarchive_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_service: ActionService<'a>,
) -> Result<ActionVisible, UnarchiveActionError> {
    let action = action_service
        .unarchive(ActionUnarchiveParams { user, action_id })
        .await?;

    Ok(ActionVisible::from(action))
}
