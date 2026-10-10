use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;
use entities::user as user_entity;
use use_cases::my_way::actions::delete::{delete_action, DeleteActionError};

use common::errors::error_responses::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[delete("/{action_id}")]
pub async fn delete_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_action(user.into_inner(), path_param.action_id, ActionService::init(&db)).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => match e {
                    DeleteActionError::NotFound(message) => response_404(message),
                    DeleteActionError::InternalServerError(_) => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
