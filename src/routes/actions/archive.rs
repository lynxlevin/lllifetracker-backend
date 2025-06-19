use ::types::ActionVisible;
use actix_web::{
    put,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Archiving an action", skip(db, user, path_param))]
#[put("/{action_id}/archive")]
pub async fn archive_action(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    path_param: Path<PathParam>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            let action = match ActionAdapter::init(&db)
                .filter_eq_user(&user)
                .get_by_id(path_param.action_id)
                .await
            {
                Ok(action) => match action {
                    Some(action) => action,
                    None => return response_404("Action with this id was not found"),
                },
                Err(e) => return response_500(e),
            };
            match ActionAdapter::init(&db).archive(action).await {
                Ok(action) => HttpResponse::Ok().json(ActionVisible::from(action)),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
