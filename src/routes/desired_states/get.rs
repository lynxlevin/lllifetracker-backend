use ::types::DesiredStateVisible;
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::desired_state_adapter::{
    DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery,
};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    desired_state_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an desired_state", skip(db, user))]
#[get("/{desired_state_id}")]
pub async fn get_desired_state(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match DesiredStateAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.desired_state_id)
                .await
            {
                Ok(desired_state) => match desired_state {
                    Some(desired_state) => {
                        HttpResponse::Ok().json(DesiredStateVisible::from(desired_state))
                    }
                    None => response_404("DesiredState with this id was not found"),
                },
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
