use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;
use entities::user as user_entity;
use use_cases::my_way::actions::unarchive::{unarchive_action, UnarchiveActionError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[put("/{action_id}/unarchive")]
pub async fn unarchive_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match unarchive_action(user.into_inner(), path_param.action_id, ActionService::init(&db)).await {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UnarchiveActionError::NotFound(message) => response_404(message),
                    UnarchiveActionError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
