use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::ActionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::actions::{types::ActionUpdateRequest, update::update_action},
    UseCaseError,
};

use crate::utils::{response_400, response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an action", skip(db, user, req, path_param))]
#[put("/{action_id}")]
pub async fn update_action_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_action(
                user.into_inner(),
                req.into_inner(),
                path_param.action_id,
                ActionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::BadRequest(message) => response_400(message),
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
