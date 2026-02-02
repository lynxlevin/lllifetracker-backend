use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::{
    direction_adapter::DirectionAdapter,
    direction_category_adapter::DirectionCategoryAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::directions::{
    create::create_direction, types::DirectionCreateRequest,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an direction", skip(db, user))]
#[post("")]
pub async fn create_direction_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_direction(
                user.into_inner(),
                req.into_inner(),
                DirectionAdapter::init(&db),
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
