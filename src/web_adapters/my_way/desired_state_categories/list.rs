use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_category_adapter::DesiredStateCategoryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::desired_state_categories::list::list_desired_state_categories;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's desired_state_categories", skip(db, user))]
#[get("")]
pub async fn list_desired_state_categories_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_desired_state_categories(
                user.into_inner(),
                DesiredStateCategoryAdapter::init(&db),
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
