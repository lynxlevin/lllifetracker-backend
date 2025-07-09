use chrono::{DateTime, FixedOffset, SubsecRound};
use sea_orm::DbErr;
use uuid::Uuid;

use crate::{
    my_way::action_tracks::types::{ActionTrackUpdateRequest, ActionTrackVisible},
    UseCaseError,
};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackLimit, ActionTrackMutation,
        ActionTrackOrder, ActionTrackQuery, UpdateActionTrackParams,
    },
    user_adapter::{UserAdapter, UserMutation},
    CustomDbErr,
    Order::Asc,
};
use entities::{action_track, user as user_entity};

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
    let original_started_at = action_track.started_at.clone();

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

    match user.first_track_at {
        Some(timestamp) => {
            if timestamp > new_action_track.started_at {
                _update_first_track_at(user_adapter, user, Some(new_action_track.started_at))
                    .await?;
            } else if timestamp == original_started_at && timestamp != new_action_track.started_at {
                let first_action_track =
                    _get_first_action_track(action_track_adapter, &user).await?;
                _update_first_track_at(user_adapter, user, Some(first_action_track.started_at))
                    .await?;
            }
        }
        None => {
            let first_action_track = _get_first_action_track(action_track_adapter, &user).await?;
            _update_first_track_at(user_adapter, user, Some(first_action_track.started_at)).await?;
        }
    }

    Ok(ActionTrackVisible::from(new_action_track))
}

async fn _get_first_action_track<'a>(
    action_track_adapter: ActionTrackAdapter<'a>,
    user: &user_entity::Model,
) -> Result<action_track::Model, UseCaseError> {
    let action_tracks = action_track_adapter
        .filter_eq_user(user)
        .filter_eq_archived_action(false)
        .order_by_started_at(Asc)
        .limit(1)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    Ok(action_tracks[0].clone())
}

async fn _update_first_track_at<'a>(
    user_adapter: UserAdapter<'a>,
    user: user_entity::Model,
    first_track_at: Option<DateTime<FixedOffset>>,
) -> Result<(), UseCaseError> {
    user_adapter
        .update_first_track_at(user, first_track_at)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
