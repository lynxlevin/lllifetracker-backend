use chrono::SubsecRound;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    my_way::action_tracks::types::{ActionTrackUpdateRequest, ActionTrackVisible},
    UseCaseError,
};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackMutation, ActionTrackQuery,
        UpdateActionTrackParams,
    },
    CustomDbErr,
};
use entities::user as user_entity;

pub async fn update_action_track<'a>(
    user: user_entity::Model,
    params: ActionTrackUpdateRequest,
    action_track_id: Uuid,
    action_track_adapter: ActionTrackAdapter<'a>,
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

    match action_track_adapter
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
    {
        Ok(action_track) => Ok(ActionTrackVisible::from(action_track)),
        Err(e) => match &e {
            DbErr::Custom(ce) => match CustomDbErr::from(ce) {
                CustomDbErr::Duplicate => Err(UseCaseError::Conflict(
                    "A track for the same action which starts at the same time exists.".to_string(),
                )),
                _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
            },
            _ => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
        },
    }
}
