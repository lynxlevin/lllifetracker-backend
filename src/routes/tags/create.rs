use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::tag_mutation::{NewTag, TagMutation};
use types::TagCreateRequest;

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
            match TagMutation::create_plain_tag(
                &db,
                NewTag {
                    name: req.name.clone(),
                    user_id: user.id,
                },
            )
            .await
            {
                Ok(tag) => HttpResponse::Created().json(tag),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
