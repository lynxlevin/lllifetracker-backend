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
use types::ActionTrackVisible;

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
            let mut action_track_query = ActionTrackQuery::init(&db)
                .filter_eq_user(&user)
                .filter_eq_archived_action(false);
            if let Some(active_only) = query.active_only {
                if active_only {
                    action_track_query = action_track_query.filter_ended_at_is_null(true);
                }
            }
            if let Some(started_at_gte) = query.started_at_gte {
                action_track_query = action_track_query.filter_started_at_gte(started_at_gte);
            }
            match action_track_query
                .order_by_started_at(Order::Desc)
                .get_all()
                .await
            {
                Ok(action_tracks) => HttpResponse::Ok().json(
                    action_tracks
                        .iter()
                        .map(|at| ActionTrackVisible::from(at))
                        .collect::<Vec<_>>(),
                ),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
