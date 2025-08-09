use crate::UseCaseError;
use chrono::{Duration, Utc};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_goal_adapter::{
        ActionGoalAdapter, ActionGoalFilter, ActionGoalMutation, ActionGoalQuery,
        UpdateActionGoalParams,
    },
};
use entities::{custom_methods::user::UserTimezoneTrait, user as user_entity};
use uuid::Uuid;

pub async fn remove_action_goal<'a>(
    user: user_entity::Model,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
    action_goal_adapter: ActionGoalAdapter<'a>,
) -> Result<(), UseCaseError> {
    let action = match action_adapter
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
    {
        Some(action) => action,
        None => return Ok(()),
    };

    let active_action_goal = action_goal_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_eq_action(&action)
        .filter_to_date_null()
        .get_one()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    match active_action_goal {
        Some(active_action_goal) => {
            let user_today = user.to_user_timezone(Utc::now()).date_naive();
            if active_action_goal.from_date == user_today {
                action_goal_adapter
                    .delete(active_action_goal)
                    .await
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
            } else {
                action_goal_adapter
                    .update(
                        UpdateActionGoalParams {
                            duration_seconds: active_action_goal.duration_seconds,
                            count: active_action_goal.count,
                            to_date: Some(user_today - Duration::days(1)),
                        },
                        active_action_goal,
                    )
                    .await
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
            }
            Ok(())
        }
        None => Ok(()),
    }
}
