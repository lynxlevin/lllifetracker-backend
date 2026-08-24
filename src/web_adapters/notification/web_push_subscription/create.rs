use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::web_push_subscription_adapter::WebPushSubscriptionAdapter;
use entities::user as user_entity;
use common::db::Db;
use use_cases::notification::web_push_subscription::{
    create::create_web_push_subscription, types::WebPushSubscriptionCreateRequest,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(skip_all)]
#[post("")]
pub async fn create_web_push_subscription_endpoint(
    db: Data<Db>,
    settings: Data<Settings>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<WebPushSubscriptionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_web_push_subscription(
                user.into_inner(),
                &settings,
                req.into_inner(),
                WebPushSubscriptionAdapter::init(&db),
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
