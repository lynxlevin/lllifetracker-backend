use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::{action_adapter::ActionAdapter, action_goal_adapter::ActionGoalAdapter};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    my_way::action_goals::{create::create_action_goal, types::ActionGoalCreateRequest},
    UseCaseError,
};

use crate::utils::{response_400, response_401, response_404, response_500};

#[tracing::instrument(name = "Creating an action goal", skip(db, user))]
#[post("")]
pub async fn create_action_goal_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionGoalCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_action_goal(
                user.into_inner(),
                req.into_inner(),
                ActionAdapter::init(&db),
                ActionGoalAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match e {
                    UseCaseError::BadRequest(message) => response_400(&message),
                    UseCaseError::NotFound(message) => response_404(&message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
