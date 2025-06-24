use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::{
    AmbitionAdapter, AmbitionFilter, AmbitionMutation, AmbitionQuery,
};
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
            let ambition = match AmbitionAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.ambition_id)
                .await
            {
                Ok(ambition) => match ambition {
                    Some(ambition) => ambition,
                    None => return HttpResponse::NoContent().into(),
                },
                Err(e) => return response_500(e),
            };
            match AmbitionAdapter::init(&db).delete(ambition).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
