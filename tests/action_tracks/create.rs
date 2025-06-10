use actix_web::{http, test, HttpMessage};
use chrono::{SubsecRound, Utc};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{sea_orm_active_enums::*, *};
use types::*;

#[actix_web::test]
async fn happy_path_time_span_type() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let started_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

    let req = test::TestRequest::post()
        .uri("/api/action_tracks")
        .set_json(ActionTrackCreateRequest {
            started_at,
            action_id: action.id,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionTrackVisible = test::read_body_json(res).await;
    assert_eq!(res.action_id, action.id);
    assert_eq!(res.started_at, started_at.trunc_subsecs(0));
    assert_eq!(res.ended_at, None);
    assert_eq!(res.duration, None);

    let action_track_in_db = action_track::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_track_in_db.user_id, user.id);
    assert_eq!(ActionTrackVisible::from(action_track_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_count_type() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id)
        .track_type(ActionTrackType::Count)
        .insert(&db)
        .await?;

    let started_at: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

    let req = test::TestRequest::post()
        .uri("/api/action_tracks")
        .set_json(ActionTrackCreateRequest {
            started_at,
            action_id: action.id,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionTrackVisible = test::read_body_json(res).await;
    assert_eq!(res.action_id, action.id);
    assert_eq!(res.started_at, started_at.trunc_subsecs(0));
    assert_eq!(res.ended_at, Some(started_at.trunc_subsecs(0)));
    assert_eq!(res.duration, Some(0));

    let action_track_in_db = action_track::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_track_in_db.user_id, user.id);
    assert_eq!(ActionTrackVisible::from(action_track_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn conflict_on_duplicate_creation() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let existing_action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::post()
        .uri("/api/action_tracks")
        .set_json(ActionTrackCreateRequest {
            started_at: existing_action_track.started_at,
            action_id: existing_action_track.action_id,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CONFLICT);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/action_tracks")
        .set_json(ActionTrackCreateRequest {
            started_at: Utc::now().into(),
            action_id: uuid::Uuid::now_v7(),
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
