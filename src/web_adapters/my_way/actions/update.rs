use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;
use entities::user as user_entity;
use use_cases::my_way::actions::{
    types::ActionUpdateRequest,
    update::{update_action, UpdateActionError},
};

use common::errors::error_responses::{response_400, response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[put("/{action_id}")]
pub async fn update_action_endpoint(
    db: Data<Db>,
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
                ActionService::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UpdateActionError::BadRequest(message) => response_400(message),
                    UpdateActionError::NotFound(message) => response_404(message),
                    UpdateActionError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
