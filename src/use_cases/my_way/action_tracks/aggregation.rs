use crate::{
    my_way::action_tracks::types::{
        ActionTrackAggregation, ActionTrackAggregationDuration, ActionTrackAggregationQuery,
    },
    UseCaseError,
};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::user as user_entity;

pub async fn aggregate_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackAggregationQuery,
    action_track_adapter: ActionTrackAdapter<'a>,
) -> Result<ActionTrackAggregation, UseCaseError> {
    let mut query = action_track_adapter.filter_eq_user(&user);
    if let Some(started_at_gte) = params.started_at_gte {
        query = query.filter_started_at_gte(started_at_gte)
    };
    if let Some(started_at_lte) = params.started_at_lte {
        query = query.filter_started_at_lte(started_at_lte)
    };
    let action_tracks = query
        .filter_ended_at_is_null(false)
        .filter_eq_archived_action(false)
        .order_by_action_id(Order::Asc)
        .order_by_started_at(Order::Desc)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<ActionTrackAggregationDuration> = vec![];
    for action_track in action_tracks {
        if res.is_empty() || res.last().unwrap().action_id != action_track.action_id {
            res.push(ActionTrackAggregationDuration {
                action_id: action_track.action_id,
                duration: action_track.duration.unwrap_or(0),
            });
        } else {
            res.last_mut().unwrap().duration += action_track.duration.unwrap_or(0)
        }
    }
    Ok(ActionTrackAggregation {
        durations_by_action: res,
    })
}
