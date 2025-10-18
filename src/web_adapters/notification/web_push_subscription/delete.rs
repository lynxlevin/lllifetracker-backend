use actix_web::{
    delete,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::web_push_subscription_adapter::WebPushSubscriptionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::notification::web_push_subscription::delete::delete_web_push_subscription;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Deleting a user's web_push_subscription.", skip(db, user))]
#[delete("")]
pub async fn delete_web_push_subscription_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_web_push_subscription(
                user.into_inner(),
                WebPushSubscriptionAdapter::init(&db),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
