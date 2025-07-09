use actix_web::{http, test, HttpMessage};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{action, action_track, user};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let original_first_track_at =
        Some(DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap());
    let user = factory::user()
        .first_track_at(original_first_track_at)
        .insert(&db)
        .await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let action_track_in_db = action_track::Entity::find_by_id(action_track.id)
        .one(&db)
        .await?;
    assert!(action_track_in_db.is_none());

    let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
    assert!(action_in_db.is_some());

    let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
    assert_eq!(user_in_db.first_track_at, original_first_track_at);

    Ok(())
}

#[actix_web::test]
async fn happy_path_update_user_first_track_at() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user()
        .first_track_at(Some(
            DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
        ))
        .insert(&db)
        .await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
    assert_eq!(user_in_db.first_track_at, None);

    Ok(())
}

#[actix_web::test]
async fn happy_path_update_user_first_track_at_switch_to_next_oldest() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user()
        .first_track_at(Some(
            DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
        ))
        .insert(&db)
        .await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap())
        .insert(&db)
        .await?;
    let second_oldest_action_track = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(DateTime::parse_from_rfc3339("2025-07-09T00:00:00Z").unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
    assert_eq!(
        user_in_db.first_track_at,
        Some(second_oldest_action_track.started_at)
    );

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
