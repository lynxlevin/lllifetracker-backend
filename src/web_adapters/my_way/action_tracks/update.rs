use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_409, response_500};
use db_adapters::{action_track_adapter::ActionTrackAdapter, user_adapter::UserAdapter};
use entities::user as user_entity;
use use_cases::{
    my_way::action_tracks::{types::ActionTrackUpdateRequest, update::update_action_track},
    UseCaseError,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_track_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an action track", skip(db, user))]
#[put("/{action_track_id}")]
pub async fn update_action_track_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionTrackUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_action_track(
                user.into_inner(),
                req.into_inner(),
                path_param.action_track_id,
                ActionTrackAdapter::init(&db),
                UserAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    UseCaseError::Conflict(message) => response_409(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
