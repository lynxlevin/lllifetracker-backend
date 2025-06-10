use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration, FixedOffset, TimeDelta};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track_0 = factory::action_track(user.id)
        .duration(Some(120))
        .action_id(action.id)
        .insert(&db)
        .await?;
    let action_track_1 = factory::action_track(user.id)
        .started_at(action_track_0.started_at + TimeDelta::seconds(1))
        .duration(Some(180))
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

    let expected = vec![
        ActionTrackVisible::from(action_track_1),
        ActionTrackVisible::from(action_track_0),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_active_only() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let _inactive_action_track = factory::action_track(user.id)
        .duration(Some(120))
        .action_id(action.id)
        .insert(&db)
        .await?;
    let active_action_track = factory::action_track(user.id)
        .started_at(_inactive_action_track.started_at + TimeDelta::seconds(1))
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks?active_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

    let expected = vec![ActionTrackVisible::from(active_action_track)];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_started_at_gte() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let started_at_gte: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2025-03-27T00:00:00Z").unwrap();
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(started_at_gte)
        .duration(Some(120))
        .insert(&db)
        .await?;
    let _old_action_track = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(started_at_gte - Duration::seconds(1))
        .duration(Some(120))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/action_tracks?started_at_gte={}",
            started_at_gte.format("%Y-%m-%dT%H:%M:%SZ")
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

    let expected = vec![ActionTrackVisible::from(action_track)];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
