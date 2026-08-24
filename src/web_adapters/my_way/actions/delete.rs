use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::action_adapter::ActionAdapter;
use entities::user as user_entity;
use use_cases::my_way::actions::delete::delete_action;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(skip(db, user))]
#[delete("/{action_id}")]
pub async fn delete_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_action(user.into_inner(), path_param.action_id, ActionAdapter::init(&db)).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
