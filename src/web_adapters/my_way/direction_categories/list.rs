use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::direction_category_adapter::DirectionCategoryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::direction_categories::list::list_direction_categories;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's direction_categories", skip(db, user))]
#[get("")]
pub async fn list_direction_categories_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_direction_categories(
                user.into_inner(),
                DirectionCategoryAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
