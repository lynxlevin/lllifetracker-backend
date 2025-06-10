use actix_web::{http, test, HttpMessage};
use chrono::{Duration, TimeDelta, Utc};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let now = Utc::now();
    let action_track_0 = factory::action_track(user.id)
        .action_id(action.id)
        .duration(Some(120))
        .insert(&db)
        .await?;
    let action_track_1 = factory::action_track(user.id)
        .action_id(action.id)
        .started_at(action_track_0.started_at + TimeDelta::seconds(1))
        .duration(Some(120))
        .insert(&db)
        .await?;
    let action_track_2 = factory::action_track(user.id)
        .duration(Some(120))
        .started_at((now - Duration::days(1)).into())
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/by_date")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<Vec<ActionTrackVisible>> = test::read_body_json(resp).await;

    let expected = vec![
        vec![
            ActionTrackVisible::from(action_track_1),
            ActionTrackVisible::from(action_track_0),
        ],
        vec![ActionTrackVisible::from(action_track_2)],
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/by_date")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
