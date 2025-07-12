use chrono::SubsecRound;
use sea_orm::DbErr;

use crate::{
    my_way::action_tracks::types::{ActionTrackCreateRequest, ActionTrackVisible},
    users::first_track_at_synchronizer::FirstTrackAtSynchronizer,
    UseCaseError,
};
use db_adapters::{
    action_adapter::{ActionAdapter, ActionFilter, ActionQuery},
    action_track_adapter::{ActionTrackAdapter, ActionTrackMutation, CreateActionTrackParams},
    user_adapter::UserAdapter,
    CustomDbErr,
};
use entities::{sea_orm_active_enums::ActionTrackType, user as user_entity};

pub async fn create_action_track<'a>(
    user: user_entity::Model,
    req: ActionTrackCreateRequest,
    action_track_adapter: ActionTrackAdapter<'a>,
    action_adapter: ActionAdapter<'a>,
    user_adapter: UserAdapter<'a>,
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
    let action_track = action_track_adapter
        .clone()
        .create(params)
        .await
        .map_err(|e| match &e {
            DbErr::Custom(message) => match CustomDbErr::from(message) {
                CustomDbErr::Duplicate => UseCaseError::Conflict(
                    "A track for the same action which starts at the same time exists.".to_string(),
                ),
                _ => UseCaseError::InternalServerError(format!("{:?}", e)),
            },
            _ => UseCaseError::InternalServerError(format!("{:?}", e)),
        })?;

    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(None, Some(action_track.clone()))
        .await?;

    Ok(ActionTrackVisible::from(action_track))
}
