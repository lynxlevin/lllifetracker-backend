use sea_orm::FromQueryResult;

use entities::web_push_subscription;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, FromQueryResult, PartialEq, Debug)]
#[sea_orm(entity = "WebPushSubscription")]
pub struct WebPushSubscriptionVisible {
    pub device_name: String,
    pub expiration_epoch_time: i64,
}

impl From<&web_push_subscription::Model> for WebPushSubscriptionVisible {
    fn from(item: &web_push_subscription::Model) -> Self {
        Self {
            device_name: item.device_name.clone(),
            expiration_epoch_time: item.expiration_epoch_time.clone(),
        }
    }
}

impl From<web_push_subscription::Model> for WebPushSubscriptionVisible {
    fn from(item: web_push_subscription::Model) -> Self {
        WebPushSubscriptionVisible::from(&item)
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct WebPushSubscriptionCreateRequest {
    pub device_name: String,
    pub endpoint: String,
    pub expiration_epoch_time: i64,
    pub p256dh_key: String,
    pub auth_key: String,
}
