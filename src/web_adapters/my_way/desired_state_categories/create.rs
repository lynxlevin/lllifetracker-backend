use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_category_adapter::DesiredStateCategoryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::desired_state_categories::{
    create::create_desired_state_category, types::DesiredStateCategoryCreateRequest,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an desired_state_category", skip(db, user))]
#[post("")]
pub async fn create_desired_state_category_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCategoryCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_desired_state_category(
                user.into_inner(),
                req.into_inner(),
                DesiredStateCategoryAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
