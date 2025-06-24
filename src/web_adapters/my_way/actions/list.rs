use actix_web::{
    get,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use sea_orm::DbConn;

use crate::utils::{response_401, response_500};
use db_adapters::action_adapter::ActionAdapter;
use entities::user as user_entity;
use use_cases::my_way::actions::{list::list_actions, types::ActionListQuery};

#[tracing::instrument(name = "Listing a user's actions", skip(db, user))]
#[get("")]
pub async fn list_actions_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ActionListQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_actions(
                user.into_inner(),
                query.into_inner(),
                ActionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
