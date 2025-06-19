use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionMutation};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an ambition", skip(db, user, path_param))]
#[delete("/{ambition_id}")]
pub async fn delete_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionAdapter::init(&db)
                .delete(path_param.ambition_id, &user)
                .await
            {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
