use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_track_service::ActionTrackService;

use common::errors::error_responses::{response_400, response_401, response_500};
use entities::user as user_entity;
use use_cases::my_way::action_tracks::{
    aggregation_daily::{aggregate_daily_action_tracks, AggregateDailyActionTracksError},
    types::ActionTrackAggregationDailyQuery,
};

#[get("/aggregation/daily")]
pub async fn aggregate_daily_action_tracks_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ActionTrackAggregationDailyQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match aggregate_daily_action_tracks(
                user.into_inner(),
                query.into_inner(),
                ActionTrackService::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    AggregateDailyActionTracksError::ValidationError(message) => response_400(&message),
                    AggregateDailyActionTracksError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
