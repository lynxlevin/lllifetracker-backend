use entities::user as user_entity;
use ::types::{self, TagVisible, INTERNAL_SERVER_ERROR_MESSAGE};
use services::tag_query::TagQuery;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

#[tracing::instrument(name = "Listing a user's tags.", skip(db, user))]
#[get("")]
pub async fn list_tags(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match TagQuery::find_all_by_user_id(&db, user.id).await {
                Ok(tags) => {
                    let res: Vec<TagVisible> =
                        tags.into_iter().map(|tag| TagVisible::from(tag)).collect();
                    HttpResponse::Ok().json(res)
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
