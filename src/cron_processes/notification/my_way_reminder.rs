use chrono::{Datelike, NaiveTime, Timelike};
use jwt_simple::reexports::rand::{seq::IteratorRandom, thread_rng};
use sea_orm::{prelude::DateTimeUtc, DbConn};
use uuid::Uuid;

use crate::notification::utils::{send_messages, Message};
use common::settings::types::Settings;
use db_adapters::{
    ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery},
    desired_state_adapter::{DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery},
    notification_rule_adapter::{
        NotificationRuleAdapter, NotificationRuleFilter, NotificationRuleOrder,
        NotificationRuleQuery,
    },
};
use entities::{notification_rule, sea_orm_active_enums::NotificationType};

enum NotificationChoice {
    Ambition,
    DesiredState,
}

pub async fn my_way_reminder(settings: &Settings, db: &DbConn, datetime: DateTimeUtc) -> () {
    let notification_rules = match get_notification_rules(db, datetime).await {
        Ok(notification_rules) => notification_rules,
        Err(_) => {
            return ();
        }
    };
    let messages = get_messages(db, notification_rules).await;
    send_messages(messages, settings, db).await;
    ()
}

// MYMEMO: test
async fn get_notification_rules(
    db: &DbConn,
    datetime: DateTimeUtc,
) -> Result<Vec<notification_rule::Model>, ()> {
    let utc_time = match NaiveTime::from_hms_opt(datetime.hour(), datetime.minute() % 10 * 10, 0) {
        Some(time) => time,
        None => {
            // MYMEMO: error log.
            // "Error on parsing datetime."
            return Err(());
        }
    };
    NotificationRuleAdapter::init(db)
        .filter_in_types(vec![
            NotificationType::AmbitionOrDesiredState,
            NotificationType::Ambition,
            NotificationType::DesiredState,
        ])
        .filter_eq_weekday(datetime.weekday())
        .filter_eq_utc_time(utc_time)
        .order_by_user_id()
        .get_all()
        .await
        .map_err(|e| {
            // MYMEMO: error log.
            // format!("Error on get_notification_rules: {:?}", e)
            ()
        })
}

async fn get_messages(
    db: &DbConn,
    notification_rules: Vec<notification_rule::Model>,
) -> Vec<Message> {
    // MYMEMO: Is there a way to reduce DB query? If not, use stream. https://users.rust-lang.org/t/how-to-use-await-inside-vec-iter-map-in-an-async-fn/65416/3
    let mut messages: Vec<Message> = vec![];
    for rule in notification_rules.iter() {
        let choice = match rule.r#type {
            NotificationType::AmbitionOrDesiredState => [
                NotificationChoice::Ambition,
                NotificationChoice::DesiredState,
            ]
            .into_iter()
            .choose(&mut thread_rng())
            .unwrap(),
            NotificationType::Ambition => NotificationChoice::Ambition,
            NotificationType::DesiredState => NotificationChoice::DesiredState,
            _ => {
                // MYMEMO: log error, this should not happen.
                continue;
            }
        };
        match get_message(choice, rule.user_id, db).await {
            Ok(message) => messages.push(message),
            Err(_) => {
                // MYMEMO: error log.
                continue;
            }
        }
    }
    messages
}

// MYMEMO: Add test
async fn get_message(
    notification_choice: NotificationChoice,
    user_id: Uuid,
    db: &DbConn,
) -> Result<Message, String> {
    let text = match notification_choice {
        NotificationChoice::Ambition => {
            let ambition = AmbitionAdapter::init(db)
                .filter_eq_user_id(user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .map_err(|e| format!("Error on get_message for user_id: {}: {:?}", user_id, e))?
                .ok_or(format!(
                    "Ambition not found on get_message for user_id: {}",
                    user_id
                ))?;
            match ambition.description {
                Some(description) => format!("{}:\n{}", ambition.name, description),
                None => ambition.name,
            }
        }
        NotificationChoice::DesiredState => {
            let desired_state = DesiredStateAdapter::init(db)
                .filter_eq_user_id(user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .map_err(|e| format!("Error on get_message for user_id: {}: {:?}", user_id, e))?
                .ok_or(format!(
                    "Ambition not found on get_message for user_id: {}",
                    user_id
                ))?;
            match desired_state.description {
                Some(description) => format!("{}:\n{}", desired_state.name, description),
                None => desired_state.name,
            }
        }
    };
    Ok(Message { text, user_id })
}
