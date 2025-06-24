use ::types::ActionVisible;
use actix_web::{
    get,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Getting an action", skip(db, user))]
#[get("/{action_id}")]
pub async fn get_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match ActionAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.action_id)
                .await
            {
                Ok(action) => match action {
                    Some(action) => HttpResponse::Ok().json(ActionVisible::from(action)),
                    None => response_404("Action with this id was not found"),
                },
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
