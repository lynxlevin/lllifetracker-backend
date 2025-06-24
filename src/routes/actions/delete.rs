use actix_web::{
    delete,
    web::{Data, Path, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::{ActionAdapter, ActionFilter, ActionMutation, ActionQuery};
use entities::user as user_entity;
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct PathParam {
    action_id: uuid::Uuid,
}

#[tracing::instrument(name = "Deleting an action", skip(db, user, path_param))]
#[delete("/{action_id}")]
pub async fn delete_action(
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
                    None => return HttpResponse::NoContent().into(),
                },
                Err(e) => return response_500(e),
            };
            match ActionAdapter::init(&db).delete(action).await {
                Ok(_) => HttpResponse::NoContent().into(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
