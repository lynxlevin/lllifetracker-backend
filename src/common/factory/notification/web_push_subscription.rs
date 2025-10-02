use entities::web_push_subscription;
use sea_orm::Set;
use uuid::Uuid;

use crate::{db::encrypt_and_encode, settings::types::Settings};

pub fn web_push_subscription(
    user_id: Uuid,
    settings: &Settings,
) -> web_push_subscription::ActiveModel {
    let encrypted_endpoint = encrypt_and_encode("endpoint".to_string(), settings).unwrap();
    let encrypted_p256dh_key = encrypt_and_encode("p256dh_key".to_string(), settings).unwrap();
    let encrypted_auth_key = encrypt_and_encode("auth_key".to_string(), settings).unwrap();
    web_push_subscription::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        device_name: Set("My device".to_string()),
        endpoint: Set(encrypted_endpoint),
        expiration_epoch_time: Set(None),
        p256dh_key: Set(encrypted_p256dh_key),
        auth_key: Set(encrypted_auth_key),
    }
}
