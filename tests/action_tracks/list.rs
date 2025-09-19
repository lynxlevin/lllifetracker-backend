use actix_web::{http, test, HttpMessage};
use chrono::{SubsecRound, TimeDelta, Utc};
use entities::action_track;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use use_cases::my_way::action_tracks::types::ActionTrackVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
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
    let Connections { app, db, .. } = init_app().await?;
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
async fn happy_path_started_at_lgte() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let now = Utc::now().trunc_subsecs(0);
    let action_tracks = (1..10)
        .map(|i| {
            factory::action_track(user.id)
                .started_at((now - TimeDelta::hours(i)).into())
                .duration(Some(i))
                .action_id(action.id)
        })
        .collect::<Vec<action_track::ActiveModel>>();
    action_track::Entity::insert_many(action_tracks)
        .exec(&db)
        .await?;
    let action_tracks = action_track::Entity::find()
        .filter(action_track::Column::ActionId.eq(action.id))
        .all(&db)
        .await?;
    let expected = &action_tracks[3..7];

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/action_tracks?started_at_gte={}&started_at_lte={}",
            expected
                .last()
                .unwrap()
                .started_at
                .format("%Y-%m-%dT%H:%M:%SZ"),
            expected
                .first()
                .unwrap()
                .started_at
                .format("%Y-%m-%dT%H:%M:%SZ"),
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionTrackVisible> = test::read_body_json(resp).await;

    assert_eq!(res.len(), expected.len());
    assert_eq!(
        res,
        expected
            .iter()
            .map(|track| ActionTrackVisible::from(track))
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
