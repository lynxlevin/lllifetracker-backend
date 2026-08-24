use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::action_adapter::ActionAdapter;
use entities::user as user_entity;
use common::db::Db;
use use_cases::my_way::actions::{create::create_action, types::ActionCreateRequest};

use crate::utils::{response_401, response_500};

#[tracing::instrument(skip(db, user))]
#[post("")]
pub async fn create_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match create_action(user.into_inner(), req.into_inner(), ActionAdapter::init(&db)).await {
            Ok(res) => HttpResponse::Created().json(res),
            Err(e) => response_500(e),
        },
        None => response_401(),
    }
}
