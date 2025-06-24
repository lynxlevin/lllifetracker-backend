use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateMutation, DesiredStateQuery,
};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an desired_state", skip(db, user, path_param))]
#[delete("/{desired_state_id}")]
pub async fn delete_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let desired_state = match DesiredStateAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.desired_state_id)
                .await
            {
                Ok(desired_state) => match desired_state {
                    Some(desired_state) => desired_state,
                    None => return HttpResponse::NoContent().into(),
                },
                Err(e) => return response_500(e),
            };
            match DesiredStateAdapter::init(&db).delete(desired_state).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
