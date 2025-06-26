use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::tag_adapter::TagAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::tags::list::list_tags;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's tags.", skip(db, user))]
#[get("")]
pub async fn list_tags_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => match list_tags(user.into_inner(), TagAdapter::init(&db)).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => response_500(e),
        },
        None => response_401(),
    }
}
