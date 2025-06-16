use ::types::{ActionTrackAggregation, ActionTrackAggregationDuration};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};

use db_adapters::{
    action_track_query::{ActionTrackQuery, ActionTrackQueryFilter, ActionTrackQueryOrder},
    Order,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;

use crate::utils::{response_401, response_500};

#[derive(Deserialize, Debug)]
struct QueryParam {
    started_at_gte: Option<chrono::DateTime<chrono::FixedOffset>>,
    started_at_lte: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[tracing::instrument(name = "Aggregating a user's action tracks", skip(db, user))]
#[get("/aggregation")]
pub async fn aggregate_action_tracks(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let action_track_query = ActionTrackQuery::init(&db);
            let mut action_track_query = action_track_query.filter_eq_user(&user);
            if let Some(started_at_gte) = query.started_at_gte {
                action_track_query = action_track_query.filter_started_at_gte(started_at_gte)
            };
            if let Some(started_at_lte) = query.started_at_lte {
                action_track_query = action_track_query.filter_started_at_lte(started_at_lte)
            };
            let action_tracks = match action_track_query
                .filter_ended_at_is_null(false)
                .filter_eq_archived_action(false)
                .order_by_action_id(Order::Asc)
                .order_by_started_at(Order::Desc)
                .get_all()
                .await
            {
                Ok(action_tracks) => action_tracks,
                Err(e) => return response_500(e),
            };
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
            HttpResponse::Ok().json(ActionTrackAggregation {
                durations_by_action: res,
            })
        }
        None => response_401(),
    }
}
