use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::action_adapter::ActionAdapter;
use entities::user as user_entity;
use use_cases::{my_way::actions::get::get_action, UseCaseError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(skip(db, user))]
#[get("/{action_id}")]
pub async fn get_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => match get_action(user.into_inner(), path_param.action_id, ActionAdapter::init(&db)).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => match &e {
                UseCaseError::NotFound(message) => response_404(message),
                _ => response_500(e),
            },
        },
        None => response_401(),
    }
}
