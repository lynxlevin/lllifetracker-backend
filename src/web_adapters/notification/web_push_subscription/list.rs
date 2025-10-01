use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::web_push_subscription_adapter::WebPushSubscriptionAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::notification::web_push_subscription::list::list_web_push_subscription;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing a user's web_push_subscription.", skip(db, user))]
#[get("")]
pub async fn list_web_push_subscription_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_web_push_subscription(
                user.into_inner(),
                WebPushSubscriptionAdapter::init(&db),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
