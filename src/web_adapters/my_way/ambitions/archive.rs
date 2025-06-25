use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::AmbitionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{my_way::ambitions::archive::archive_ambition, UseCaseError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    ambition_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an ambition", skip(db, user, path_param))]
#[put("/{ambition_id}/archive")]
pub async fn archive_ambition_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match archive_ambition(
                user.into_inner(),
                path_param.ambition_id,
                AmbitionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
