use std::collections::HashMap;

use common::settings::types::Settings;
use db_adapters::web_push_subscription_adapter::{
    WebPushSubscriptionAdapter, WebPushSubscriptionFilter, WebPushSubscriptionMutation,
    WebPushSubscriptionQuery,
};
use entities::web_push_subscription;
use sea_orm::DbConn;
use tracing::{event, instrument, Level};
use uuid::Uuid;

use crate::notification::utils::web_push_messenger::{WebPushMessenger, WebPushMessengerResult};

mod web_push_messenger;

#[derive(Debug)]
pub struct Message {
    pub text: String,
    pub user_id: Uuid,
}

// MYMEMO: This should have a wrapper error type for all the errors in this file.
// MYMEMO: nice to have a test, but to do that, need to create a messenger_builder to DI.
#[instrument(skip_all)]
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
        Err(e) => {
            event!(Level::ERROR, %e);
            return ();
        }
    };

    let message_len = messages.len();
    for message in messages {
        send_web_push(
            message,
            &mut web_push_subscriptions_by_user_id,
            settings,
            db,
        )
        .await;
    }
    event!(Level::INFO, "Successfully sent {} messages", message_len);
    ()
}

#[instrument(skip_all)]
async fn send_web_push(
    message: Message,
    web_push_subscriptions_by_user_id: &mut HashMap<Uuid, web_push_subscription::Model>,
    settings: &Settings,
    db: &DbConn,
) -> () {
    let subscription = match web_push_subscriptions_by_user_id.remove(&message.user_id) {
        Some(subscription) => subscription,
        None => {
            event!(Level::WARN, "No web_push_subscription found.");
            return ();
        }
    };

    let messenger = match WebPushMessenger::new(&subscription, settings) {
        Ok(messenger) => messenger,
        Err(e) => {
            event!(Level::ERROR, "Error on initializing WebPushMessenger: {e}");
            return ();
        }
    };
    match messenger.send_message(&message.text).await {
        Ok(result) => match result {
            WebPushMessengerResult::OK => {
                web_push_subscriptions_by_user_id.insert(subscription.user_id, subscription);
            }
            WebPushMessengerResult::InvalidSubscription => {
                event!(Level::WARN, "WebPushMessengerResult::InvalidSubscription for this message. Deleting the web_push_subscription.");

                // NOTE: iOS returns 201 even when it's unsubscribed.
                if let Err(e) = WebPushSubscriptionAdapter::init(db)
                    .delete(subscription)
                    .await
                {
                    event!(Level::ERROR, "Error on deleting web_push_subscription: {e}")
                }
            }
        },
        Err(e) => {
            event!(Level::ERROR, "Error on send_message: {e}");
            return ();
        }
    };
}
