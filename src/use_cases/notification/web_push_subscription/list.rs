use db_adapters::web_push_subscription_adapter::{
    WebPushSubscriptionAdapter, WebPushSubscriptionQuery,
};
use entities::user as user_entity;

use crate::{notification::web_push_subscription::types::WebPushSubscriptionVisible, UseCaseError};

pub async fn list_web_push_subscription<'a>(
    user: user_entity::Model,
    web_push_subscription_adapter: WebPushSubscriptionAdapter<'a>,
) -> Result<Option<WebPushSubscriptionVisible>, UseCaseError> {
    web_push_subscription_adapter
        .get_by_user(&user)
        .await
        .map(|subscription| subscription.map(|sub| WebPushSubscriptionVisible::from(sub)))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
