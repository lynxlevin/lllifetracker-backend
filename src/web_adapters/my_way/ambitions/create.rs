use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::AmbitionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::my_way::ambitions::{create::create_ambition, types::AmbitionCreateRequest};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an ambition", skip(db, user))]
#[post("")]
pub async fn create_ambition_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<AmbitionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_ambition(
                user.into_inner(),
                req.into_inner(),
                AmbitionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
