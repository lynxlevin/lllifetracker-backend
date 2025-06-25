use actix_web::{
    put,
    web::{Data, Json, Path, ReqData},
    HttpResponse,
};
use db_adapters::{
    desired_state_adapter::DesiredStateAdapter,
    desired_state_category_adapter::DesiredStateCategoryAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::desired_states::{types::DesiredStateUpdateRequest, update::update_desired_state},
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Updating an desired_state", skip(db, user, req, path_param))]
#[put("/{desired_state_id}")]
pub async fn update_desired_state_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<DesiredStateUpdateRequest>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match update_desired_state(
                user.into_inner(),
                req.into_inner(),
                path_param.desired_state_id,
                DesiredStateAdapter::init(&db),
                DesiredStateCategoryAdapter::init(&db),
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
