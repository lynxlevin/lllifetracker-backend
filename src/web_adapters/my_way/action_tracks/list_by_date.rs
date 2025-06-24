use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};
use db_adapters::action_track_adapter::ActionTrackAdapter;
use entities::user as user_entity;
use use_cases::my_way::action_tracks::list_by_date::list_action_tracks_by_date;

#[tracing::instrument(name = "Listing a user's action tracks by date", skip(db, user))]
#[get("/by_date")]
pub async fn list_action_tracks_by_date_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_action_tracks_by_date(user.into_inner(), ActionTrackAdapter::init(&db)).await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
