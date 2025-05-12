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
    active_only: Option<bool>,
    started_at_gte: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[tracing::instrument(name = "Listing a user's action tracks", skip(db, user))]
#[get("")]
pub async fn list_action_tracks(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<QueryParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let mut filters = ActionTrackQuery::get_default_filters();
            filters.show_inactive = !query.active_only.unwrap_or(false);
            filters.started_at_gte = query.started_at_gte;
            match ActionTrackQuery::find_by_user_id_with_filters(&db, user.id, filters).await {
                Ok(action_tracks) => HttpResponse::Ok().json(action_tracks),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
