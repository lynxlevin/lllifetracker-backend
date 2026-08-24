use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use common::db::Db;

use crate::utils::{response_401, response_500};
use db_adapters::action_track_adapter::ActionTrackAdapter;
use entities::user as user_entity;
use use_cases::my_way::action_tracks::{list::list_action_tracks, types::ActionTrackListQuery};

#[tracing::instrument(skip(db, user))]
#[get("")]
pub async fn list_action_tracks_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ActionTrackListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_action_tracks(user.into_inner(), query.into_inner(), ActionTrackAdapter::init(&db)).await {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
