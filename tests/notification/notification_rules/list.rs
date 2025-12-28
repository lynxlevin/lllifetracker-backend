use actix_web::{http, test, HttpMessage};
use chrono::NaiveTime;
use entities::sea_orm_active_enums::NotificationType;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::notification::notification_rule::types::{NotificationRuleVisible, RecurrenceType};

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path_utc_plus_9() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    factory::create_everyday_rules(
        user.id,
        &db,
        NotificationType::Ambition,
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    )
    .await?;
    factory::create_weekday_rules(
        user.id,
        &db,
        NotificationType::AmbitionOrDesiredState,
        NaiveTime::from_hms_opt(23, 10, 0).unwrap(),
        true,
    )
    .await?;
    factory::create_weekend_rules(
        user.id,
        &db,
        NotificationType::DesiredState,
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        true,
    )
    .await?;

    let req = test::TestRequest::get()
        .uri("/api/notification_rules")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<NotificationRuleVisible> = test::read_body_json(resp).await;
    let expected = vec![
        NotificationRuleVisible {
            r#type: NotificationType::Ambition,
            recurrence_type: RecurrenceType::Everyday,
            time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        },
        NotificationRuleVisible {
            r#type: NotificationType::AmbitionOrDesiredState,
            recurrence_type: RecurrenceType::Weekday,
            time: NaiveTime::from_hms_opt(8, 10, 0).unwrap(),
        },
        NotificationRuleVisible {
            r#type: NotificationType::DesiredState,
            recurrence_type: RecurrenceType::Weekend,
            time: NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        },
    ];

    assert_eq!(body, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/notification_rules")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
