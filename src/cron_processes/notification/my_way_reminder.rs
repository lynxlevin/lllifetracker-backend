use chrono::{Datelike, NaiveTime, Timelike};
use jwt_simple::reexports::rand::{seq::IteratorRandom, thread_rng};
use sea_orm::{prelude::DateTimeUtc, DbConn};

use crate::notification::utils::web_push_sender::{send_messages, Message};
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
    let messages = match get_messages(db, datetime).await {
        Ok(messages) => messages,
        Err(e) => {
            // MYMEMO: error log.
            return ();
        }
    };
    send_messages(messages, settings, db).await;
    ()
}

async fn get_messages(db: &DbConn, datetime: DateTimeUtc) -> Result<Vec<Message>, impl ToString> {
    let utc_time = match NaiveTime::from_hms_opt(datetime.hour(), datetime.minute() % 10 * 10, 0) {
        Some(time) => time,
        None => {
            // MYMEMO: return error message.
            return Err("");
        }
    };
    let notification_rules = match NotificationRuleAdapter::init(db)
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
    {
        Ok(rules) => rules,
        Err(_) => {
            // MYMEMO: return error message.
            return Err("");
        }
    };

    // MYMEMO: Is there a way to reduce DB query? If not, use stream. https://users.rust-lang.org/t/how-to-use-await-inside-vec-iter-map-in-an-async-fn/65416/3
    let mut messages: Vec<Message> = vec![];
    for rule in notification_rules.iter() {
        match get_message(rule, db).await {
            Some(message) => messages.push(message),
            None => (),
        }
    }
    Ok(messages)
}

async fn get_message(notification_rule: &notification_rule::Model, db: &DbConn) -> Option<Message> {
    let choice = match notification_rule.r#type {
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
            return None;
        }
    };
    let text = match choice {
        NotificationChoice::Ambition => {
            let ambition = match AmbitionAdapter::init(db)
                .filter_eq_user_id(notification_rule.user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .unwrap_or_else(|_| {
                    // MYMEMO: error log
                    None
                }) {
                Some(ambition) => ambition,
                None => return None,
            };
            match ambition.description {
                Some(description) => format!("{}:\n{}", ambition.name, description),
                None => ambition.name,
            }
        }
        NotificationChoice::DesiredState => {
            let desired_state = match DesiredStateAdapter::init(db)
                .filter_eq_user_id(notification_rule.user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .unwrap_or_else(|_| {
                    // MYMEMO: error log
                    None
                }) {
                Some(desired_state) => desired_state,
                None => return None,
            };
            match desired_state.description {
                Some(description) => format!("{}:\n{}", desired_state.name, description),
                None => desired_state.name,
            }
        }
    };

    Some(Message {
        text,
        user_id: notification_rule.user_id,
    })
}
