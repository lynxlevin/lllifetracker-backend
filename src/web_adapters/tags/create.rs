use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::TagAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::tags::{create::create_plain_tag, types::TagCreateRequest};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating a plain tag", skip(db, user))]
#[post("/plain")]
pub async fn create_plain_tag_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<TagCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_plain_tag(user.into_inner(), req.into_inner(), TagAdapter::init(&db)).await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
