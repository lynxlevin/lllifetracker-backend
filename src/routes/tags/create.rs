use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::{CreatePlainTagParams, TagAdapter, TagMutation};
use entities::user as user_entity;
use sea_orm::DbConn;
use types::{TagCreateRequest, TagType, TagVisible};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating a plain tag", skip(db, user))]
#[post("/plain")]
pub async fn create_plain_tag(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<TagCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match TagAdapter::init(&db)
                .create_plain(CreatePlainTagParams {
                    name: req.name.clone(),
                    user_id: user.id,
                })
                .await
            {
                Ok(tag) => HttpResponse::Created().json(TagVisible {
                    id: tag.id,
                    name: tag.name.unwrap(),
                    tag_type: TagType::Plain,
                    created_at: tag.created_at,
                }),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
