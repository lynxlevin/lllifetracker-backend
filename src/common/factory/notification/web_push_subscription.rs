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

pub trait WebPushSubscriptionFactory {
    fn encrypt_and_save_p256dh_key(
        self,
        p256dh_key: String,
        settings: &Settings,
    ) -> web_push_subscription::ActiveModel;
    fn encrypt_and_save_auth_key(
        self,
        auth_key: String,
        settings: &Settings,
    ) -> web_push_subscription::ActiveModel;
    fn get_model(self) -> web_push_subscription::Model;
}

impl WebPushSubscriptionFactory for web_push_subscription::ActiveModel {
    fn encrypt_and_save_p256dh_key(
        mut self,
        p256dh_key: String,
        settings: &Settings,
    ) -> web_push_subscription::ActiveModel {
        self.p256dh_key = Set(encrypt_and_encode(p256dh_key, settings).unwrap());
        self
    }
    fn encrypt_and_save_auth_key(
        mut self,
        auth_key: String,
        settings: &Settings,
    ) -> web_push_subscription::ActiveModel {
        self.auth_key = Set(encrypt_and_encode(auth_key, settings).unwrap());
        self
    }
    fn get_model(self) -> web_push_subscription::Model {
        web_push_subscription::Model {
            id: self.id.unwrap(),
            user_id: self.user_id.unwrap(),
            device_name: self.device_name.unwrap(),
            endpoint: self.endpoint.unwrap(),
            expiration_epoch_time: self.expiration_epoch_time.unwrap(),
            p256dh_key: self.p256dh_key.unwrap(),
            auth_key: self.auth_key.unwrap(),
        }
    }
}
