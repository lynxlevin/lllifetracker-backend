use db_adapters::web_push_subscription_adapter::{
    CreateWebPushSubscriptionParams, WebPushSubscriptionAdapter, WebPushSubscriptionMutation,
};
use entities::user as user_entity;

use crate::{
    notification::web_push_subscription::types::{
        WebPushSubscriptionCreateRequest, WebPushSubscriptionVisible,
    },
    UseCaseError,
};

pub async fn create_web_push_subscription<'a>(
    user: user_entity::Model,
    params: WebPushSubscriptionCreateRequest,
    web_push_subscription_adapter: WebPushSubscriptionAdapter<'a>,
) -> Result<WebPushSubscriptionVisible, UseCaseError> {
    web_push_subscription_adapter
        .create(CreateWebPushSubscriptionParams {
            user_id: user.id,
            device_name: params.device_name.clone(),
            endpoint: params.endpoint.clone(),
            expiration_epoch_time: params.expiration_epoch_time,
            p256dh_key: params.p256dh_key.clone(),
            auth_key: params.auth_key.clone(),
        })
        .await
        .map(|subscription| WebPushSubscriptionVisible::from(subscription))
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
