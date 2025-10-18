use db_adapters::web_push_subscription_adapter::{
    WebPushSubscriptionAdapter, WebPushSubscriptionMutation, WebPushSubscriptionQuery,
};
use entities::user as user_entity;

use crate::UseCaseError;

pub async fn delete_web_push_subscription<'a>(
    user: user_entity::Model,
    web_push_subscription_adapter: WebPushSubscriptionAdapter<'a>,
) -> Result<(), UseCaseError> {
    let subscription = web_push_subscription_adapter
        .clone()
        .get_by_user(&user)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    if let Some(subscription) = subscription {
        web_push_subscription_adapter
            .delete(subscription)
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    }
    Ok(())
}
