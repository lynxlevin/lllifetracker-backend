use domain_services::action_service::{
    ActionDeleteParams, ActionService, ActionServiceError, ActionServiceMutation,
};
use thiserror::Error;
use uuid::Uuid;

use entities::user as user_entity;

#[derive(Debug, Error)]
pub enum DeleteActionError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for DeleteActionError {
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
pub async fn delete_action<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_service: ActionService<'a>,
) -> Result<(), DeleteActionError> {
    action_service
        .delete(ActionDeleteParams { user, action_id })
        .await
        .map_err(|e| e.into())
}
