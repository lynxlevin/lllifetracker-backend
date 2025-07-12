use chrono::SubsecRound;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    my_way::action_tracks::types::{ActionTrackUpdateRequest, ActionTrackVisible},
    users::first_track_at_synchronizer::FirstTrackAtSynchronizer,
    UseCaseError,
};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackMutation, ActionTrackQuery,
        UpdateActionTrackParams,
    },
    user_adapter::UserAdapter,
    CustomDbErr,
};
use entities::user as user_entity;

pub async fn update_action_track<'a>(
    user: user_entity::Model,
    params: ActionTrackUpdateRequest,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
) -> Result<ActionTrackVisible, UseCaseError> {
    let action_track = action_track_adapter
        .clone()
        .filter_eq_user(&user)
        .get_by_id(action_track_id)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "ActionTrack with this id was not found".to_string(),
        ))?;
    let old_action_track = action_track.clone();

    let new_action_track = action_track_adapter
        .clone()
        .update(
            action_track,
            UpdateActionTrackParams {
                started_at: params.started_at.trunc_subsecs(0),
                ended_at: params
                    .ended_at
                    .and_then(|ended_at| Some(ended_at.trunc_subsecs(0))),
                duration: params.ended_at.and_then(|ended_at| {
                    Some(
                        (ended_at.trunc_subsecs(0) - params.started_at.trunc_subsecs(0))
                            .num_seconds(),
                    )
                }),
                action_id: params.action_id,
            },
        )
        .await
        .map_err(|e| match &e {
            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                CustomDbErr::Duplicate => UseCaseError::Conflict(
                    "A track for the same action which starts at the same time exists.".to_string(),
                ),
                _ => UseCaseError::InternalServerError(format!("{:?}", e)),
            },
            _ => UseCaseError::InternalServerError(format!("{:?}", e)),
        })?;

    FirstTrackAtSynchronizer::init(action_track_adapter, user_adapter, user)
        .update_user_first_track_at(Some(old_action_track), Some(new_action_track.clone()))
        .await?;

    Ok(ActionTrackVisible::from(new_action_track))
}
