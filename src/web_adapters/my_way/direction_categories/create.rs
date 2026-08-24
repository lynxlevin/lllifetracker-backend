use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::direction_category_adapter::DirectionCategoryAdapter;
use entities::user as user_entity;
use use_cases::my_way::direction_categories::{
    create::create_direction_category, types::DirectionCategoryCreateRequest,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(skip(db, user))]
#[post("")]
pub async fn create_direction_category_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionCategoryCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_direction_category(
                user.into_inner(),
                req.into_inner(),
                DirectionCategoryAdapter::init(&db),
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
