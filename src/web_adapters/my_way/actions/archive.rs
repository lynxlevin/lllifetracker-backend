use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    action_adapter::ActionAdapter, action_track_adapter::ActionTrackAdapter,
    user_adapter::UserAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{my_way::actions::archive::archive_action, UseCaseError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an action", skip(db, user, path_param))]
#[put("/{action_id}/archive")]
pub async fn archive_action_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match archive_action(
                user.into_inner(),
                path_param.action_id,
                ActionAdapter::init(&db),
                UserAdapter::init(&db),
                ActionTrackAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
