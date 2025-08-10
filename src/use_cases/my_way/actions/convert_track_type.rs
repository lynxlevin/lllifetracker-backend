use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    my_way::actions::types::{ActionTrackTypeConversionRequest, ActionVisible},
    UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery},
    action_goal_adapter::{
        ActionGoalAdapter, ActionGoalFilter, ActionGoalMutation, ActionGoalQuery,
        UpdateActionGoalParams,
    },
};
use entities::{custom_methods::user::UserTimezoneTrait, user as user_entity};

pub async fn convert_action_track_type<'a>(
    user: user_entity::Model,
    params: ActionTrackTypeConversionRequest,
    action_id: Uuid,
    action_adapter: ActionAdapter<'a>,
    action_goal_adapter: ActionGoalAdapter<'a>,
) -> Result<ActionVisible, UseCaseError> {
    let action = action_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "Action with this id was not found".to_string(),
        ))?;

    if action.track_type == params.track_type {
        return Ok(ActionVisible::from(action));
    }

    let active_action_goal = action_goal_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_eq_action(&action)
        .filter_to_date_null()
        .get_one()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if let Some(active_action_goal) = active_action_goal {
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
                        to_date: Some(user_today - Duration::days(1)),
                        duration_seconds: active_action_goal.duration_seconds,
                        count: active_action_goal.count,
                    },
                    active_action_goal,
                )
                .await
                .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
        }
    }

    action_adapter
        .convert_track_type(action, params.track_type.clone())
        .await
        .map(|action| ActionVisible::from(action))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
