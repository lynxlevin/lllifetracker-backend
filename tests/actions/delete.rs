use actix_web::{http, test, HttpMessage};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{action, tag, user};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let (action, tag) = factory::action(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/actions/{}", action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

    let action_in_db = action::Entity::find_by_id(action.id).one(&db.db).await?;
    assert!(action_in_db.is_none());

    let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db.db).await?;
    assert!(tag_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn happy_path_update_user_first_track_at() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user()
        .first_track_at(Some(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap()))
        .insert(&db.db)
        .await?;
    let actions = factory::create_actions(
        vec![
            ActionParam { name: "action_with_oldest_track", ..Default::default() },
            ActionParam { name: "action_with_second_oldest_track", ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let action_tracks = factory::create_action_tracks(
        vec![
            ActionTrackParam {
                name: "oldest_track",
                action_id: actions.get("action_with_oldest_track").unwrap().id,
                started_at: DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
                duration: None,
            },
            ActionTrackParam {
                name: "second_oldest_track",
                action_id: actions.get("action_with_second_oldest_track").unwrap().id,
                started_at: DateTime::parse_from_rfc3339("2025-07-09T00:00:00Z").unwrap(),
                duration: None,
            },
        ],
        &user,
        &db,
    )
    .await?;

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/actions/{}",
            actions.get("action_with_oldest_track").unwrap().id
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let user_in_db = user::Entity::find_by_id(user.id).one(&db.db).await?.unwrap();
    assert_eq!(
        user_in_db.first_track_at,
        Some(action_tracks.get("second_oldest_track").unwrap().started_at)
    );

    Ok(())
}

#[actix_web::test]
async fn not_found_on_wrong_id() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    factory::action(user.id).insert_with_tag(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/actions/{}", Uuid::now_v7()))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let action = factory::action(user.id).insert(&db.db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/actions/{}", action.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
