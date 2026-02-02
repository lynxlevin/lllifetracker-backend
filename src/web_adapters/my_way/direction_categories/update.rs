use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::direction_category_adapter::DirectionCategoryAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::direction_categories::{
        types::DirectionCategoryUpdateRequest, update::update_direction_category,
    },
    UseCaseError,
};
use uuid::Uuid;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    category_id: Uuid,
}

#[tracing::instrument(name = "Updating an direction_category", skip(db, user))]
#[put("/{category_id}")]
pub async fn update_direction_category_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionCategoryUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_direction_category(
                user.into_inner(),
                req.into_inner(),
                path_param.category_id,
                DirectionCategoryAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
