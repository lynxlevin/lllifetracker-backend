use std::collections::HashMap;

use common::settings::types::Settings;
use db_adapters::web_push_subscription_adapter::{
    WebPushSubscriptionAdapter, WebPushSubscriptionFilter, WebPushSubscriptionMutation,
    WebPushSubscriptionQuery,
};
use sea_orm::DbConn;
use uuid::Uuid;

use crate::notification::utils::web_push_messenger::{WebPushMessenger, WebPushMessengerResult};

mod web_push_messenger;

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

        let builder = match WebPushMessenger::new(&subscription, settings) {
            Ok(builder) => builder,
            Err(_) => {
                // MYMEMO: error log.
                return ();
            }
        };
        match builder.send_message(message.text).await {
            Ok(result) => match result {
                WebPushMessengerResult::OK => {
                    web_push_subscriptions_by_user_id.insert(subscription.user_id, subscription);
                }
                WebPushMessengerResult::InvalidSubscription => {
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
            },
            Err(e) => {
                // MYMEMO: error log.
                continue;
            }
        };
    }
    ()
}
