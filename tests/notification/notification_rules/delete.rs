use actix_web::{http, test, HttpMessage};
use chrono::NaiveTime;
use entities::{
    notification_rule::{Column, Entity},
    sea_orm_active_enums::NotificationType,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use crate::utils::{init_app, Connections};
use common::factory;

#[actix_web::test]
async fn happy_path_delete_ambition() -> Result<(), DbErr> {
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
        NotificationType::AmbitionOrDirection,
        NaiveTime::from_hms_opt(23, 10, 0).unwrap(),
        true,
    )
    .await?;
    factory::create_weekend_rules(
        user.id,
        &db,
        NotificationType::Direction,
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        true,
    )
    .await?;

    let req = test::TestRequest::delete()
        .uri("/api/notification_rules?type=Ambition")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition.len(), 0);

    let rules_in_db_ambition_or_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::AmbitionOrDirection))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition_or_direction.len(), 5);

    let rules_in_db_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Direction))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_direction.len(), 2);

    Ok(())
}

#[actix_web::test]
async fn happy_path_delete_ambition_or_direction() -> Result<(), DbErr> {
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
        NotificationType::AmbitionOrDirection,
        NaiveTime::from_hms_opt(23, 10, 0).unwrap(),
        true,
    )
    .await?;
    factory::create_weekend_rules(
        user.id,
        &db,
        NotificationType::Direction,
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        true,
    )
    .await?;

    let req = test::TestRequest::delete()
        .uri("/api/notification_rules?type=AmbitionOrDirection")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition.len(), 7);

    let rules_in_db_ambition_or_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::AmbitionOrDirection))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition_or_direction.len(), 0);

    let rules_in_db_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Direction))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_direction.len(), 2);

    Ok(())
}

#[actix_web::test]
async fn happy_path_delete_direction() -> Result<(), DbErr> {
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
        NotificationType::AmbitionOrDirection,
        NaiveTime::from_hms_opt(23, 10, 0).unwrap(),
        true,
    )
    .await?;
    factory::create_weekend_rules(
        user.id,
        &db,
        NotificationType::Direction,
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        true,
    )
    .await?;

    let req = test::TestRequest::delete()
        .uri("/api/notification_rules?type=Direction")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let rules_in_db_ambition = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Ambition))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition.len(), 7);

    let rules_in_db_ambition_or_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::AmbitionOrDirection))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_ambition_or_direction.len(), 5);

    let rules_in_db_direction = Entity::find()
        .filter(Column::UserId.eq(user.id))
        .filter(Column::Type.eq(NotificationType::Direction))
        .all(&db)
        .await?;
    assert_eq!(rules_in_db_direction.len(), 0);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::delete()
        .uri("/api/notification_rules?type=Ambition")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
