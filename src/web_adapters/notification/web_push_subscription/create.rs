use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::web_push_subscription_adapter::WebPushSubscriptionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::{
    notification::web_push_subscription::{
        create::create_web_push_subscription, types::WebPushSubscriptionCreateRequest,
    },
    UseCaseError,
};

use crate::utils::{response_401, response_404, response_500};

#[tracing::instrument(name = "Registering a web push subscription", skip(db, user))]
#[post("")]
pub async fn create_web_push_subscription_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<WebPushSubscriptionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_web_push_subscription(
                user.into_inner(),
                req.into_inner(),
                WebPushSubscriptionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => match &e {
                    UseCaseError::NotFound(message) => response_404(message),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
