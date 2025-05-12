use ::types::ActionTrackVisible;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use chrono::{FixedOffset, TimeZone};
use entities::{sea_orm_active_enums::TimezoneEnum, user as user_entity};
use sea_orm::DbConn;
use services::action_track_query::ActionTrackQuery;

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
            match ActionTrackQuery::find_by_user_id_with_filters(
                &db,
                user.id,
                ActionTrackQuery::get_default_filters(),
            )
            .await
            {
                Ok(action_tracks) => {
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
                            res.push(vec![action_track])
                        } else {
                            res.last_mut().unwrap().push(action_track);
                        }
                    }
                    HttpResponse::Ok().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}

fn started_on_same_day<Tz2: TimeZone>(
    date_1: &ActionTrackVisible,
    date_2: &ActionTrackVisible,
    user_timezone: &Tz2,
) -> bool {
    date_1.started_at.with_timezone(user_timezone).date_naive()
        == date_2.started_at.with_timezone(user_timezone).date_naive()
}
