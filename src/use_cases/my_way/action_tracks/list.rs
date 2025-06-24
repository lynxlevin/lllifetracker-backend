use crate::{
    my_way::action_tracks::types::{ActionTrackListQuery, ActionTrackVisible},
    UseCaseError,
};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::user as user_entity;

pub async fn list_action_tracks<'a>(
    user: user_entity::Model,
    params: ActionTrackListQuery,
    action_track_adapter: ActionTrackAdapter<'a>,
) -> Result<Vec<ActionTrackVisible>, UseCaseError> {
    let mut query = action_track_adapter
        .filter_eq_user(&user)
        .filter_eq_archived_action(false);
    if let Some(active_only) = params.active_only {
        if active_only {
            query = query.filter_ended_at_is_null(true);
        }
    }
    if let Some(started_at_gte) = params.started_at_gte {
        query = query.filter_started_at_gte(started_at_gte);
    }
    let action_tracks = query
        .order_by_started_at(Order::Desc)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    Ok(action_tracks
        .iter()
        .map(|at| ActionTrackVisible::from(at))
        .collect::<Vec<_>>())
}
