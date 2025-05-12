use ::types::{ActionTrackAggregation, ActionTrackAggregationDuration};
use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use serde::Deserialize;
use services::action_track_query::ActionTrackQuery;

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
            let mut filters = ActionTrackQuery::get_default_filters();
            filters.started_at_gte = query.started_at_gte;
            filters.started_at_lte = query.started_at_lte;
            filters.show_active = false;
            filters.for_daily_aggregation = true;
            match ActionTrackQuery::find_by_user_id_with_filters(&db, user.id, filters).await {
                Ok(action_tracks) => {
                    let mut res: Vec<ActionTrackAggregationDuration> = vec![];
                    for action_track in action_tracks {
                        if res.is_empty() || res.last().unwrap().action_id != action_track.action_id
                        {
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
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
