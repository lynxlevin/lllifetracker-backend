use chrono::{NaiveTime, Weekday};
use jwt_simple::reexports::rand::{seq::IteratorRandom, thread_rng};
use sea_orm::DbConn;
use tracing::{event, instrument, Level};
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

#[derive(Debug)]
enum NotificationChoice {
    Ambition,
    DesiredState,
}

#[instrument(skip_all)]
pub async fn my_way_reminder(
    settings: &Settings,
    db: &DbConn,
    weekday: Weekday,
    utc_time: NaiveTime,
) -> () {
    let notification_rules = match get_notification_rules(db, weekday, utc_time).await {
        Ok(notification_rules) => notification_rules,
        Err(_) => {
            return ();
        }
    };
    event!(
        Level::INFO,
        "Will process {} notification_rules",
        notification_rules.len()
    );
    let messages = get_messages(db, notification_rules).await;
    event!(Level::INFO, "Will process {} messages", messages.len());
    send_messages(messages, settings, db).await;
    event!(Level::INFO, "Finishing my_way_reminder.");
    ()
}

#[instrument(skip_all)]
async fn get_notification_rules(
    db: &DbConn,
    weekday: Weekday,
    utc_time: NaiveTime,
) -> Result<Vec<notification_rule::Model>, ()> {
    NotificationRuleAdapter::init(db)
        .filter_in_types(vec![
            NotificationType::AmbitionOrDesiredState,
            NotificationType::Ambition,
            NotificationType::DesiredState,
        ])
        .filter_eq_weekday(weekday)
        .filter_eq_utc_time(utc_time)
        .order_by_user_id()
        .get_all()
        .await
        .map_err(|e| {
            event!(Level::ERROR, %e);
            ()
        })
}

#[instrument(skip_all)]
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
                event!(
                    Level::ERROR,
                    "This type of notification should not be passed to this function. type: {:?}",
                    rule.r#type
                );
                continue;
            }
        };
        match get_random_message(&choice, rule.user_id, db).await {
            Some(message) => messages.push(message),
            None => {
                if rule.r#type == NotificationType::AmbitionOrDesiredState {
                    event!(
                        Level::INFO,
                        "type is AmbitionOrDesiredState, falling back to another choice."
                    );
                    let choice = match choice {
                        NotificationChoice::Ambition => NotificationChoice::DesiredState,
                        NotificationChoice::DesiredState => NotificationChoice::Ambition,
                    };
                    match get_random_message(&choice, rule.user_id, db).await {
                        Some(message) => messages.push(message),
                        None => (),
                    }
                }
            }
        }
    }
    messages
}

#[instrument(skip(db))]
async fn get_random_message(
    notification_choice: &NotificationChoice,
    user_id: Uuid,
    db: &DbConn,
) -> Option<Message> {
    let text = match notification_choice {
        NotificationChoice::Ambition => {
            let ambition = match AmbitionAdapter::init(db)
                .filter_eq_user_id(user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .unwrap_or_else(|e| {
                    event!(Level::ERROR, %e);
                    return None;
                }) {
                Some(ambition) => ambition,
                None => {
                    event!(Level::WARN, "Ambition not found.");
                    return None;
                }
            };
            match ambition.description {
                Some(description) => format!("{}:\n{}", ambition.name, description),
                None => ambition.name,
            }
        }
        NotificationChoice::DesiredState => {
            let desired_state = match DesiredStateAdapter::init(db)
                .filter_eq_user_id(user_id)
                .filter_eq_archived(false)
                .get_random()
                .await
                .unwrap_or_else(|e| {
                    event!(Level::ERROR, %e);
                    return None;
                }) {
                Some(ambition) => ambition,
                None => {
                    event!(Level::WARN, "DesiredState not found.");
                    return None;
                }
            };
            match desired_state.description {
                Some(description) => format!("{}:\n{}", desired_state.name, description),
                None => desired_state.name,
            }
        }
    };
    Some(Message { text, user_id })
}

#[cfg(test)]
mod tests {
    use common::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };
    use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

    use super::*;

    fn default_notification_rule(
        user_id: Uuid,
        weekday: Weekday,
        time: NaiveTime,
    ) -> notification_rule::ActiveModel {
        factory::notification_rule(user_id)
            .r#type(NotificationType::Ambition)
            .weekday(weekday)
            .utc_time(time)
    }

    #[actix_web::test]
    async fn test_get_notification_rules() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let weekday = Weekday::Mon;
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        // FIXME: This has to be done in a better way.
        notification_rule::Entity::delete_many().exec(&db).await?;
        let notification_rule_0 = default_notification_rule(user.id, weekday.clone(), time.clone())
            .insert(&db)
            .await?;
        let notification_rule_1 = default_notification_rule(user.id, weekday.clone(), time.clone())
            .r#type(NotificationType::DesiredState)
            .insert(&db)
            .await?;
        let notification_rule_2 = default_notification_rule(user.id, weekday.clone(), time.clone())
            .r#type(NotificationType::AmbitionOrDesiredState)
            .insert(&db)
            .await?;
        let no_use_notification_rule_0 =
            default_notification_rule(user.id, weekday.clone(), time.clone())
                .r#type(NotificationType::Action);
        let no_use_notification_rule_1 =
            default_notification_rule(user.id, weekday.clone(), time.clone())
                .weekday(weekday.succ());
        let no_use_notification_rule_2 =
            default_notification_rule(user.id, weekday.clone(), time.clone())
                .weekday(weekday.pred());
        let no_use_notification_rule_3 =
            default_notification_rule(user.id, weekday.clone(), time.clone())
                .utc_time(NaiveTime::from_hms_opt(0, 10, 0).unwrap());
        let no_use_notification_rule_4 =
            default_notification_rule(user.id, weekday.clone(), time.clone())
                .utc_time(NaiveTime::from_hms_opt(1, 0, 0).unwrap());
        notification_rule::Entity::insert_many([
            no_use_notification_rule_0,
            no_use_notification_rule_1,
            no_use_notification_rule_2,
            no_use_notification_rule_3,
            no_use_notification_rule_4,
        ])
        .exec(&db)
        .await?;

        let res = get_notification_rules(&db, weekday, time).await;
        assert!(res.is_ok());
        let res = res.unwrap();

        assert_eq!(res.len(), 3);
        assert!(res.contains(&notification_rule_0));
        assert!(res.contains(&notification_rule_1));
        assert!(res.contains(&notification_rule_2));

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_random_message_case_ambition_no_description() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let _desired_state = factory::desired_state(user.id).insert(&db).await?;

        let res = get_random_message(&NotificationChoice::Ambition, user.id, &db).await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(res.text, ambition.name);
        assert_eq!(res.user_id, user.id);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_random_message_case_ambition_with_description() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id)
            .description(Some("Description".to_string()))
            .insert(&db)
            .await?;
        let _desired_state = factory::desired_state(user.id).insert(&db).await?;

        let res = get_random_message(&NotificationChoice::Ambition, user.id, &db).await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(
            res.text,
            format!("{}:\n{}", ambition.name, ambition.description.unwrap())
        );
        assert_eq!(res.user_id, user.id);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_random_message_case_desired_state_no_description() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let _ambition = factory::ambition(user.id).insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let res = get_random_message(&NotificationChoice::DesiredState, user.id, &db).await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(res.text, desired_state.name);
        assert_eq!(res.user_id, user.id);

        Ok(())
    }

    #[actix_web::test]
    async fn test_get_random_message_case_desired_state_with_description() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await;
        let user = factory::user().insert(&db).await?;
        let _ambition = factory::ambition(user.id).insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .description(Some("Description".to_string()))
            .insert(&db)
            .await?;

        let res = get_random_message(&NotificationChoice::DesiredState, user.id, &db).await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(
            res.text,
            format!(
                "{}:\n{}",
                desired_state.name,
                desired_state.description.unwrap()
            )
        );
        assert_eq!(res.user_id, user.id);

        Ok(())
    }
}
