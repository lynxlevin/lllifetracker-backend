use ::types::AmbitionVisible;
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an ambition", skip(db, user))]
#[get("/{ambition_id}")]
pub async fn get_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.ambition_id)
                .await
            {
                Ok(ambition) => match ambition {
                    Some(ambition) => HttpResponse::Ok().json(AmbitionVisible::from(ambition)),
                    None => response_404("Ambition with this id was not found"),
                },
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
