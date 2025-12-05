use actix_web::{http, test, HttpMessage};
use chrono::{
    NaiveTime,
    Weekday::{self, Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use entities::{
    notification_rule::{Column, Entity},
    sea_orm_active_enums::NotificationType,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use use_cases::notification::notification_rule::types::{
    NotificationRuleCreateRequest, RecurrenceType,
};

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path_everyday_utc_plus_9() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = NotificationRuleCreateRequest {
        r#type: NotificationType::Ambition,
        recurrence_type: RecurrenceType::Everyday,
        time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/notification_rules")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .filter(Column::ActionId.is_null())
        .order_by_asc(Column::Weekday)
        .all(&db)
        .await?;
    let expected_utc_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    for rule in &rules_in_db_ambition {
        assert_eq!(rule.utc_time, expected_utc_time);
    }
    let weekdays = rules_in_db_ambition
        .iter()
        .map(|rule| Weekday::try_from(rule.weekday as u8).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(weekdays.as_slice(), [Mon, Tue, Wed, Thu, Fri, Sat, Sun]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_weekday_utc_plus_9() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = NotificationRuleCreateRequest {
        r#type: NotificationType::Ambition,
        recurrence_type: RecurrenceType::Weekday,
        time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/notification_rules")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .filter(Column::ActionId.is_null())
        .order_by_asc(Column::Weekday)
        .all(&db)
        .await?;
    let expected_utc_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    for rule in &rules_in_db_ambition {
        assert_eq!(rule.utc_time, expected_utc_time);
    }
    let weekdays = rules_in_db_ambition
        .iter()
        .map(|rule| Weekday::try_from(rule.weekday as u8).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(weekdays.as_slice(), [Mon, Tue, Wed, Thu, Sun]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_weekend_utc_plus_9() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;

    let req_body = NotificationRuleCreateRequest {
        r#type: NotificationType::Ambition,
        recurrence_type: RecurrenceType::Weekend,
        time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
    };

    let req = test::TestRequest::post()
        .set_json(req_body.clone())
        .uri("/api/notification_rules")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::CREATED);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .filter(Column::ActionId.is_null())
        .order_by_asc(Column::Weekday)
        .all(&db)
        .await?;
    let expected_utc_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    for rule in &rules_in_db_ambition {
        assert_eq!(rule.utc_time, expected_utc_time);
    }
    let weekdays = rules_in_db_ambition
        .iter()
        .map(|rule| Weekday::try_from(rule.weekday as u8).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(weekdays.as_slice(), [Fri, Sat]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::delete()
        .set_json(NotificationRuleCreateRequest {
            r#type: NotificationType::Ambition,
            recurrence_type: RecurrenceType::Everyday,
            time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        })
        .uri("/api/notification_rules?type=Ambition")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
