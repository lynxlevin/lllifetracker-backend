use ::types::TagVisible;
use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use services::tag_query::TagQuery;

use crate::utils::{response_401, response_500};

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
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
