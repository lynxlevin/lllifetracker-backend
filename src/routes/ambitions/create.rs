use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::ambition_adapter::{AmbitionAdapter, AmbitionMutation, CreateAmbitionParams};
use entities::user as user_entity;
use sea_orm::DbConn;
use types::{AmbitionCreateRequest, AmbitionVisible};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating an ambition", skip(db, user))]
#[post("")]
pub async fn create_ambition(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<AmbitionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            match AmbitionAdapter::init(&db)
                .create_with_tag(CreateAmbitionParams {
                    name: req.name.clone(),
                    description: req.description.clone(),
                    user_id: user.id,
                })
                .await
            {
                Ok(ambition) => HttpResponse::Created().json(AmbitionVisible::from(ambition)),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
