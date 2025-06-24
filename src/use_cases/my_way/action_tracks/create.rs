use chrono::SubsecRound;
use sea_orm::DbErr;

use crate::{
    my_way::action_tracks::types::{ActionTrackCreateRequest, ActionTrackVisible},
    UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_track_adapter::{ActionTrackAdapter, ActionTrackMutation, CreateActionTrackParams},
    CustomDbErr,
};
use entities::{sea_orm_active_enums::ActionTrackType, user as user_entity};

pub async fn create_action_track<'a>(
    user: user_entity::Model,
    req: ActionTrackCreateRequest,
    action_track_adapter: ActionTrackAdapter<'a>,
    action_adapter: ActionAdapter<'a>,
) -> Result<ActionTrackVisible, UseCaseError> {
    let action = action_adapter
        .filter_eq_user(&user)
        .get_by_id(req.action_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "An action with that id does not exist.".to_string(),
        ))?;

    let params = match action.track_type {
        ActionTrackType::TimeSpan => CreateActionTrackParams {
            started_at: req.started_at.trunc_subsecs(0),
            ended_at: None,
            duration: None,
            action_id: req.action_id,
            user_id: user.id,
        },
        ActionTrackType::Count => CreateActionTrackParams {
            started_at: req.started_at.trunc_subsecs(0),
            ended_at: Some(req.started_at.trunc_subsecs(0)),
            duration: Some(0),
            action_id: req.action_id,
            user_id: user.id,
        },
    };
    match action_track_adapter.create(params).await {
        Ok(action_track) => Ok(ActionTrackVisible::from(action_track)),
        Err(e) => match &e {
            DbErr::Custom(message) => match CustomDbErr::from(message) {
                CustomDbErr::Duplicate => Err(UseCaseError::Conflict(
                    "A track for the same action which starts at the same time exists.".to_string(),
                )),
                _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            },
            _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    }
}
