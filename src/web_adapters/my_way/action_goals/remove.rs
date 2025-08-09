use actix_web::{
    delete,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::{action_adapter::ActionAdapter, action_goal_adapter::ActionGoalAdapter};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::action_goals::{remove::remove_action_goal, types::ActionGoalRemoveQuery};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Removing an action goal", skip(db, user))]
#[delete("")]
pub async fn remove_action_goal_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<ActionGoalRemoveQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match remove_action_goal(
                user.into_inner(),
                query.action_id,
                ActionAdapter::init(&db),
                ActionGoalAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => match e {
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
