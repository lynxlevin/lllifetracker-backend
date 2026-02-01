use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::direction_adapter::DirectionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{my_way::directions::unarchive::unarchive_direction, UseCaseError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    direction_id: uuid::Uuid,
}

#[tracing::instrument(
    name = "Restoring an direction from unarchive",
    skip(db, user, path_param)
)]
#[put("/{direction_id}/unarchive")]
pub async fn unarchive_direction_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match unarchive_direction(
                user.into_inner(),
                path_param.direction_id,
                DirectionAdapter::init(&db),
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
