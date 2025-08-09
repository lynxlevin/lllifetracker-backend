use crate::{
    my_way::action_goals::types::{ActionGoalCreateRequest, ActionGoalVisible},
    UseCaseError,
};
use chrono::{Duration, Utc};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_goal_adapter::{
        ActionGoalAdapter, ActionGoalFilter, ActionGoalMutation, ActionGoalQuery,
        CreateActionGoalParams, UpdateActionGoalParams,
    },
};
use entities::{
    action, custom_methods::user::UserTimezoneTrait, sea_orm_active_enums::ActionTrackType,
    user as user_entity,
};

pub async fn create_action_goal<'a>(
    user: user_entity::Model,
    params: ActionGoalCreateRequest,
    action_adapter: ActionAdapter<'a>,
    action_goal_adapter: ActionGoalAdapter<'a>,
) -> Result<ActionGoalVisible, UseCaseError> {
    let action = action_adapter
        .filter_eq_user(&user)
        .get_by_id(params.action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let (parsed_params, action) = _parse_params(params, action, &user)?;

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
            if active_action_goal.from_date == parsed_params.from_date {
                action_goal_adapter
                    .update(
                        UpdateActionGoalParams {
                            duration_seconds: parsed_params.duration_seconds,
                            count: parsed_params.count,
                            to_date: None,
                        },
                        active_action_goal,
                    )
                    .await
                    .map(|action_goal| ActionGoalVisible::from(action_goal))
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
            } else {
                action_goal_adapter
                    .clone()
                    .update(
                        UpdateActionGoalParams {
                            duration_seconds: active_action_goal.duration_seconds,
                            count: active_action_goal.count,
                            to_date: Some(parsed_params.from_date - Duration::days(1)),
                        },
                        active_action_goal,
                    )
                    .await
                    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
                _create_action_goal(action_goal_adapter, parsed_params).await
            }
        }
        None => _create_action_goal(action_goal_adapter, parsed_params).await,
    }
}

fn _parse_params(
    params: ActionGoalCreateRequest,
    action: Option<action::Model>,
    user: &user_entity::Model,
) -> Result<(CreateActionGoalParams, action::Model), UseCaseError> {
    let action = action.ok_or(UseCaseError::NotFound("This action not found.".to_string()))?;
    match action.track_type {
        ActionTrackType::TimeSpan => {
            if params.duration_seconds.is_none() || params.count.is_some() {
                return Err(UseCaseError::BadRequest(
                    "duration_seconds cannot be empty".to_string(),
                ));
            }
        }
        ActionTrackType::Count => {
            if params.count.is_none() || params.duration_seconds.is_some() {
                return Err(UseCaseError::BadRequest(
                    "count cannot be empty".to_string(),
                ));
            }
        }
    }

    let user_today = user.to_user_timezone(Utc::now()).date_naive();

    Ok((
        CreateActionGoalParams {
            from_date: user_today,
            duration_seconds: params.duration_seconds,
            count: params.count,
            action_id: params.action_id,
            user_id: user.id,
        },
        action,
    ))
}

async fn _create_action_goal<'a>(
    action_goal_adapter: ActionGoalAdapter<'a>,
    params: CreateActionGoalParams,
) -> Result<ActionGoalVisible, UseCaseError> {
    action_goal_adapter
        .create(params)
        .await
        .map(|action_goal| ActionGoalVisible::from(action_goal))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
