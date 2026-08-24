use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use common::db::Db;
use db_adapters::direction_category_adapter::DirectionCategoryAdapter;
use entities::user as user_entity;
use use_cases::my_way::direction_categories::delete::delete_direction_category;
use uuid::Uuid;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    category_id: Uuid,
}

#[tracing::instrument(skip(db, user))]
#[delete("/{category_id}")]
pub async fn delete_direction_category_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_direction_category(
                user.into_inner(),
                path_param.category_id,
                DirectionCategoryAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
