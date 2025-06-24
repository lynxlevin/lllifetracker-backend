use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_category_adapter::{
    CreateDesiredStateCategoryParams, DesiredStateCategoryAdapter, DesiredStateCategoryMutation,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use types::{DesiredStateCategoryCreateRequest, DesiredStateCategoryVisible};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an desired_state_category", skip(db, user))]
#[post("")]
pub async fn create_desired_state_category(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateCategoryCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateCategoryAdapter::init(&db)
                .create(CreateDesiredStateCategoryParams {
                    name: req.name.clone(),
                    user_id: user.id,
                })
                .await
            {
                Ok(category) => {
                    let res: DesiredStateCategoryVisible = category.into();
                    HttpResponse::Created().json(res)
                }
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
