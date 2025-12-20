use awc::http::StatusCode;
use sea_orm::DbConn;
use std::collections::HashMap;
use uuid::Uuid;

use crate::notification::utils::web_push_request_builder::WebPushRequestBuilder;
use common::settings::types::Settings;
use db_adapters::web_push_subscription_adapter::{
    WebPushSubscriptionAdapter, WebPushSubscriptionFilter, WebPushSubscriptionMutation,
    WebPushSubscriptionQuery,
};
use entities::web_push_subscription;

pub struct Message {
    pub text: String,
    pub user_id: Uuid,
}

pub async fn send_messages(messages: Vec<Message>, settings: &Settings, db: &DbConn) -> () {
    let mut user_ids = messages
        .iter()
        .map(|message| message.user_id)
        .collect::<Vec<_>>();
    user_ids.dedup();

    let mut web_push_subscriptions_by_user_id = match WebPushSubscriptionAdapter::init(db)
        .filter_in_user_ids(user_ids)
        .get_all()
        .await
    {
        Ok(subs) => subs.into_iter().fold(HashMap::new(), |mut acc, sub| {
            acc.insert(sub.user_id, sub);
            acc
        }),
        Err(_) => {
            // MYMEMO: error log.
            return ();
        }
    };

    for message in messages {
        let subscription = match web_push_subscriptions_by_user_id.remove(&message.user_id) {
            Some(subscription) => subscription,
            None => {
                // MYMEMO: error log.
                continue;
            }
        };

        let result_status_code = match send_message(message.text, &subscription, settings).await {
            Ok(result_status_code) => result_status_code,
            Err(e) => {
                // MYMEMO: error log.
                continue;
            }
        };
        match result_status_code {
            StatusCode::NOT_FOUND | StatusCode::GONE => {
                // MYMEMO: error log.
                // format!("The WebPushSubscription with id: {} for user_id: {} is invalid.", subscription.id, subscription.user_id)

                // NOTE: iOS returns 201 even when it's unsubscribed.
                if let Err(e) = WebPushSubscriptionAdapter::init(db)
                    .delete(subscription)
                    .await
                {
                    // MYMEMO: error log
                }
            }
            _ => {
                web_push_subscriptions_by_user_id.insert(subscription.user_id, subscription);
            }
        }
    }
    ()
}

async fn send_message(
    message: String,
    subscription: &web_push_subscription::Model,
    settings: &Settings,
) -> Result<StatusCode, impl ToString> {
    let builder = WebPushRequestBuilder::new(subscription, settings).map_err(|e| e.to_string())?;
    let encrypted_message = builder
        .encrypt_message(message)
        .map_err(|e| e.to_string())?;
    let request = builder.get_awc_client(None).map_err(|e| e.to_string())?;
    request
        .send_body(encrypted_message)
        .await
        .map(|res| res.status())
        .map_err(|e| e.to_string())
}
