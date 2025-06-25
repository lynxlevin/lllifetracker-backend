use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::DesiredStateAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{my_way::desired_states::get::get_desired_state, UseCaseError};

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an desired_state", skip(db, user))]
#[get("/{desired_state_id}")]
pub async fn get_desired_state_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match get_desired_state(
                user.into_inner(),
                path_param.desired_state_id,
                DesiredStateAdapter::init(&db),
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
