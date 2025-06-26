use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::TagAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    tags::{types::TagUpdateRequest, update::update_plain_tag},
    UseCaseError,
};

use crate::utils::{response_400, response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    tag_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating a plain tag", skip(db, user))]
#[put("/plain/{tag_id}")]
pub async fn update_plain_tag_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<TagUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_plain_tag(
                user.into_inner(),
                req.into_inner(),
                path_param.tag_id,
                TagAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::BadRequest(message) => response_400(message),
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
