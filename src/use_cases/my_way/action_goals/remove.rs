use domain_services::action_goal_service::{
    ActionGoalDiscontinueParams, ActionGoalService, ActionGoalServiceError, ActionGoalServiceMutation,
};
use entities::user as user_entity;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RemoveActionGoalError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionGoalServiceError> for RemoveActionGoalError {
    fn from(e: ActionGoalServiceError) -> Self {
        match e {
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
pub async fn remove_action_goal<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_goal_service: ActionGoalService<'a>,
) -> Result<(), RemoveActionGoalError> {
    match action_goal_service
        .discontinue(ActionGoalDiscontinueParams { user: &user, action_id })
        .await
    {
        Ok(_) => {}
        Err(e) => match e {
            ActionGoalServiceError::ActionNotFound() | ActionGoalServiceError::ActionGoalNotFound() => {
                return Ok(());
            }
            _ => return Err(RemoveActionGoalError::InternalServerError(e.to_string())),
        },
    };

    Ok(())
}
