use crate::my_way::actions::types::ActionVisibleWithGoal;
use domain_services::action_service::{ActionService, ActionServiceError, ActionServiceQuery};
use entities::user as user_entity;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListActionError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<ActionServiceError> for ListActionError {
    fn from(e: ActionServiceError) -> Self {
        match e {
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
    ),
    skip_all
)]
pub async fn list_actions<'a>(
    user: user_entity::Model,
    action_service: ActionService<'a>,
) -> Result<Vec<ActionVisibleWithGoal>, ListActionError> {
    let actions = action_service.list(user.id).await?;

    Ok(actions
        .into_iter()
        .map(|(action, goal)| ActionVisibleWithGoal::from(action).with_goal(goal))
        .collect())
}
