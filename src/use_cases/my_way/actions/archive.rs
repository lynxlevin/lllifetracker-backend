use domain_services::action_service::{
    ActionArchiveParams, ActionService, ActionServiceError, ActionServiceMutation,
};
use thiserror::Error;
use uuid::Uuid;

use crate::my_way::actions::types::ActionVisible;
use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum ArchiveActionError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for ArchiveActionError {
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
pub async fn archive_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_service: ActionService<'a>,
) -> Result<ActionVisible, ArchiveActionError> {
    let action = action_service.archive(ActionArchiveParams { user, action_id }).await?;

    Ok(ActionVisible::from(action))
}
