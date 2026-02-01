use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    direction_adapter::DirectionAdapter,
    direction_category_adapter::DirectionCategoryAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::directions::{types::DirectionUpdateRequest, update::update_direction},
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    direction_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an direction", skip(db, user, req, path_param))]
#[put("/{direction_id}")]
pub async fn update_direction_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DirectionUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_direction(
                user.into_inner(),
                req.into_inner(),
                path_param.direction_id,
                DirectionAdapter::init(&db),
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
