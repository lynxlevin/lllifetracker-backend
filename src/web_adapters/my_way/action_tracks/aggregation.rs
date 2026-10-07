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
    aggregation::{aggregate_action_tracks, AggregateActionTracksError},
    types::ActionTrackAggregationQuery,
};

#[get("/aggregation")]
pub async fn aggregate_action_tracks_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ActionTrackAggregationQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match aggregate_action_tracks(user.into_inner(), query.into_inner(), ActionTrackService::init(&db))
                .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    AggregateActionTracksError::ValidationError(message) => response_400(&message),
                    AggregateActionTracksError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
