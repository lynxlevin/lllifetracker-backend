use crate::{
    my_way::action_goals::types::{ActionGoalCreateRequest, ActionGoalVisible},
    UseCaseError,
};
use db_adapters::action_goal_adapter::{
    ActionGoalAdapter, ActionGoalMutation, CreateActionGoalParams,
};
use entities::user as user_entity;

pub async fn create_action_goal<'a>(
    user: user_entity::Model,
    params: ActionGoalCreateRequest,
    action_goal_adapter: ActionGoalAdapter<'a>,
) -> Result<ActionGoalVisible, UseCaseError> {
    action_goal_adapter
        .create(CreateActionGoalParams {
            from_date: params.from_date,
            duration_seconds: params.duration_seconds,
            count: params.count,
            action_id: params.action_id,
            user_id: user.id,
        })
        .await
        .map(|action_goal| ActionGoalVisible::from(action_goal))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
