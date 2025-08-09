use crate::{
    my_way::action_goals::types::{ActionGoalCreateRequest, ActionGoalVisible},
    UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_goal_adapter::{ActionGoalAdapter, ActionGoalMutation, CreateActionGoalParams},
};
use entities::{action, sea_orm_active_enums::ActionTrackType, user as user_entity};

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

    let parsed_params = _parse_params(params, action, &user)?;

    action_goal_adapter
        .create(parsed_params)
        .await
        .map(|action_goal| ActionGoalVisible::from(action_goal))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn _parse_params(
    params: ActionGoalCreateRequest,
    action: Option<action::Model>,
    user: &user_entity::Model,
) -> Result<CreateActionGoalParams, UseCaseError> {
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

    Ok(CreateActionGoalParams {
        from_date: params.from_date,
        duration_seconds: params.duration_seconds,
        count: params.count,
        action_id: params.action_id,
        user_id: user.id,
    })
}
