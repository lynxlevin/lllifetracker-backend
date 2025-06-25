use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::TagAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{tags::delete::delete_plain_tag, UseCaseError};

use crate::utils::{response_400, response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    tag_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting a plain tag", skip(db, user))]
#[delete("/plain/{tag_id}")]
pub async fn delete_plain_tag_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_plain_tag(user.into_inner(), path_param.tag_id, TagAdapter::init(&db))
                .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => match &e {
                    UseCaseError::BadRequest(message) => response_400(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
