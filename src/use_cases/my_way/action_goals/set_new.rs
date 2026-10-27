use crate::my_way::action_goals::types::{ActionGoalSetNewRequest, ActionGoalVisible};
use domain_services::action_goal_service::{
    ActionGoalService, ActionGoalServiceError, ActionGoalServiceMutation, ActionGoalSetNewParams,
};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetNewActionGoalError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionGoalServiceError> for SetNewActionGoalError {
    fn from(e: ActionGoalServiceError) -> Self {
        match e {
            ActionGoalServiceError::ActionNotFound() => Self::NotFound(e.to_string()),
            ActionGoalServiceError::DurationSecondsNullForTimeSpanAction()
            | ActionGoalServiceError::CountNotNullForTimeSpanAction()
            | ActionGoalServiceError::CountNullForCountAction()
            | ActionGoalServiceError::DurationSecondsNotNullForCountAction() => {
                Self::ValidationError(e.to_string())
            }
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        params.action_id = params.action_id.to_string(),
        params.duration_seconds = params.duration_seconds,
        params.count = params.count,
    ),
    skip_all
)]
pub async fn set_new_action_goal<'a>(
    user: user_entity::Model,
    params: ActionGoalSetNewRequest,
    action_goal_service: ActionGoalService<'a>,
) -> Result<ActionGoalVisible, SetNewActionGoalError> {
    let action_goal = action_goal_service
        .set_new(ActionGoalSetNewParams {
            user: &user,
            action_id: params.action_id,
            duration_seconds: params.duration_seconds,
            count: params.count,
        })
        .await?;

    Ok(ActionGoalVisible::from(action_goal))
}
