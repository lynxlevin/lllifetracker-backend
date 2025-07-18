use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::DesiredStateAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::desired_states::delete::delete_desired_state;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an desired_state", skip(db, user, path_param))]
#[delete("/{desired_state_id}")]
pub async fn delete_desired_state_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_desired_state(
                user.into_inner(),
                path_param.desired_state_id,
                DesiredStateAdapter::init(&db),
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
