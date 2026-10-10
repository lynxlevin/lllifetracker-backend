use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;
use entities::user as user_entity;
use use_cases::my_way::actions::{
    create::{create_action, CreateActionError},
    types::ActionCreateRequest,
};

use common::errors::error_responses::{response_401, response_500};

#[post("")]
pub async fn create_action_endpoint(
    db: Data<Db>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<ActionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => match create_action(user.into_inner(), req.into_inner(), ActionService::init(&db)).await {
            Ok(res) => HttpResponse::Created().json(res),
            Err(e) => match e {
                CreateActionError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
