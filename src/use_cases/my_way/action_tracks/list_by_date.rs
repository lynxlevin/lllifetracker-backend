use chrono::{FixedOffset, TimeZone};

use crate::{my_way::action_tracks::types::ActionTrackVisible, UseCaseError};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::{action_track, sea_orm_active_enums::TimezoneEnum, user as user_entity};

pub async fn list_action_tracks_by_date<'a>(
    user: user_entity::Model,
    action_track_adapter: ActionTrackAdapter<'a>,
) -> Result<Vec<Vec<ActionTrackVisible>>, UseCaseError> {
    let action_tracks = action_track_adapter
        .filter_eq_user(&user)
        .filter_eq_archived_action(false)
        .order_by_started_at(Order::Desc)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let mut res: Vec<Vec<ActionTrackVisible>> = vec![];
    let user_offset = match user.timezone {
        TimezoneEnum::Utc => FixedOffset::east_opt(0).unwrap(),
        TimezoneEnum::AsiaTokyo => FixedOffset::east_opt(9 * 3600).unwrap(),
    };
    for action_track in action_tracks {
        if res.is_empty()
            || !started_on_same_day(
                res.last().unwrap().last().unwrap(),
                &action_track,
                &user_offset,
            )
        {
            res.push(vec![ActionTrackVisible::from(action_track)])
        } else {
            res.last_mut()
                .unwrap()
                .push(ActionTrackVisible::from(action_track));
        }
    }
    Ok(res)
}

fn started_on_same_day<Tz2: TimeZone>(
    date_1: &ActionTrackVisible,
    date_2: &action_track::Model,
    user_timezone: &Tz2,
) -> bool {
    date_1.started_at.with_timezone(user_timezone).date_naive()
        == date_2.started_at.with_timezone(user_timezone).date_naive()
}
