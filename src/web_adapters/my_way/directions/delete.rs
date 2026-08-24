use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::direction_adapter::DirectionAdapter;
use entities::user as user_entity;
use common::db::Db;
use use_cases::my_way::directions::delete::delete_direction;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    direction_id: uuid::Uuid,
}

#[tracing::instrument(skip(db, user))]
#[delete("/{direction_id}")]
pub async fn delete_direction_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_direction(user.into_inner(), path_param.direction_id, DirectionAdapter::init(&db)).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
