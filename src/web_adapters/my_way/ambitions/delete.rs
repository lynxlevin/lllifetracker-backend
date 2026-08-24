use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::ambition_adapter::AmbitionAdapter;
use entities::user as user_entity;
use use_cases::my_way::ambitions::delete::delete_ambition;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(skip(db, user))]
#[delete("/{ambition_id}")]
pub async fn delete_ambition_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_ambition(user.into_inner(), path_param.ambition_id, AmbitionAdapter::init(&db)).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
