use actix_web::{
    post,
    web::{Data, ReqData},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::{
    ambition_adapter::AmbitionAdapter, desired_state_adapter::DesiredStateAdapter,
    web_push_subscription_adapter::WebPushSubscriptionAdapter,
};
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{notification::web_push_subscription::send::send_web_push, UseCaseError};

use crate::utils::{response_401, response_404, response_410, response_500};

#[tracing::instrument(name = "Sending web push notification", skip(db, user, settings))]
#[post("/send")]
pub async fn send_web_push_endpoint(
    db: Data<DbConn>,
    settings: Data<Settings>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match send_web_push(
                user.into_inner(),
                &settings,
                WebPushSubscriptionAdapter::init(&db),
                AmbitionAdapter::init(&db),
                DesiredStateAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::Accepted().finish(),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    UseCaseError::Gone => response_410(),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
