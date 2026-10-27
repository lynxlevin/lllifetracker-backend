use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use common::db::Db;
use domain_services::action_service::ActionService;

use common::errors::error_responses::{response_401, response_500};
use entities::user as user_entity;
use use_cases::my_way::actions::list::{list_actions, ListActionError};

#[get("")]
pub async fn list_actions_endpoint(db: Data<Db>, user: Option<ReqData<user_entity::Model>>) -> HttpResponse {
    match user {
        Some(user) => match list_actions(user.into_inner(), ActionService::init(&db)).await {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(e) => match e {
                ListActionError::InternalServerError(_) => response_500(e),
            },
        },
        None => response_401(),
    }
}
