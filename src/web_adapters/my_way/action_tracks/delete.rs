use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};
use db_adapters::action_track_adapter::ActionTrackAdapter;
use entities::user as user_entity;
use use_cases::my_way::action_tracks::delete::delete_action_track;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action track", skip(db, user))]
#[delete("/{action_track_id}")]
pub async fn delete_action_track_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_action_track(
                user.into_inner(),
                path_param.action_track_id,
                ActionTrackAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
