use ::types::ActionTrackVisible;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use chrono::{FixedOffset, TimeZone};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackOrder, ActionTrackQuery,
    },
    Order,
};
use entities::{action_track, sea_orm_active_enums::TimezoneEnum, user as user_entity};
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's action tracks by date", skip(db, user))]
#[get("/by_date")]
pub async fn list_action_tracks_by_date(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let action_tracks = match ActionTrackAdapter::init(&db)
                .filter_eq_user(&user)
                .filter_eq_archived_action(false)
                .order_by_started_at(Order::Desc)
                .get_all()
                .await
            {
                Ok(action_tracks) => action_tracks,
                Err(e) => return response_500(e),
            };
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
            HttpResponse::Ok().json(res)
        }
        None => response_401(),
    }
}

fn started_on_same_day<Tz2: TimeZone>(
    date_1: &ActionTrackVisible,
    date_2: &action_track::Model,
    user_timezone: &Tz2,
) -> bool {
    date_1.started_at.with_timezone(user_timezone).date_naive()
        == date_2.started_at.with_timezone(user_timezone).date_naive()
}
