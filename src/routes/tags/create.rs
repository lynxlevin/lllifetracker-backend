use ::types::{self, INTERNAL_SERVER_ERROR_MESSAGE};
use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::tag_mutation::{NewTag, TagMutation};
use types::TagCreateRequest;

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
                Ok(tag) => {
                    HttpResponse::Created().json(tag)
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed on DB query: {:#?}", e);
                    HttpResponse::InternalServerError().json(types::ErrorResponse {
                        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                    })
                }
            }
        }
        None => HttpResponse::Unauthorized().json("You are not logged in."),
    }
}
