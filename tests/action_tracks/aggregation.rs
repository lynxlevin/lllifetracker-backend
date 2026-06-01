use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::action_tracks::types::{ActionTrackAggregation, ActionTrackAggregationDuration};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let actions = create_actions(
        vec![
            ActionParam { name: "action_0".to_string(), ..Default::default() },
            ActionParam { name: "action_1".to_string(), ..Default::default() },
            ActionParam { name: "_action_2".to_string(), ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let action_0 = actions.get("action_0").unwrap();
    let action_1 = actions.get("action_1").unwrap();
    let jst_now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
    let action_tracks = create_action_tracks(
        vec![
            ActionTrackParam {
                name: "action_0_track_0".to_string(),
                action_id: action_0.id,
                started_at: jst_now - Duration::days(1),
                duration: Some(120),
            },
            ActionTrackParam {
                name: "_action_0_track_1".to_string(),
                action_id: action_0.id,
                started_at: jst_now,
                duration: Some(180),
            },
            ActionTrackParam {
                name: "action_0_track_2".to_string(),
                action_id: action_0.id,
                started_at: jst_now + Duration::days(1),
                duration: Some(300),
            },
            ActionTrackParam {
                name: "_action_0_track_3".to_string(),
                action_id: action_0.id,
                started_at: jst_now + Duration::days(2),
                duration: Some(550),
            },
            ActionTrackParam {
                name: "action_1_track_0".to_string(),
                action_id: action_1.id,
                started_at: jst_now + Duration::days(1),
                duration: Some(350),
            },
        ],
        &user,
        &db,
    )
    .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/action_tracks/aggregation?dates={},{}",
            (jst_now - Duration::days(1)).date_naive().format("%Y%m%d"),
            (jst_now + Duration::days(1)).date_naive().format("%Y%m%d"),
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: ActionTrackAggregation = test::read_body_json(resp).await;

    let expected = ActionTrackAggregation {
        durations_by_action: vec![
            ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_tracks.get("action_0_track_0").unwrap().duration.unwrap()
                    + action_tracks.get("action_0_track_2").unwrap().duration.unwrap(),
                count: 2,
            },
            ActionTrackAggregationDuration {
                action_id: action_1.id,
                duration: action_tracks.get("action_1_track_0").unwrap().duration.unwrap(),
                count: 1,
            },
        ],
    };

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn started_at_gte_lte() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let query_started_at_gte: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2025-01-27T00:00:00Z").unwrap();
    let query_started_at_lte: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2025-01-27T23:59:59Z").unwrap();
    let actions = create_actions(
        vec![
            ActionParam { name: "action_0".to_string(), ..Default::default() },
            ActionParam { name: "action_1".to_string(), ..Default::default() },
            ActionParam { name: "_action_2".to_string(), ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let action_0 = actions.get("action_0").unwrap();
    let action_1 = actions.get("action_1").unwrap();
    let action_tracks = create_action_tracks(
        vec![
            ActionTrackParam {
                name: "_action_0_track_0".to_string(),
                action_id: action_0.id,
                started_at: query_started_at_gte - Duration::seconds(1),
                duration: Some(120),
            },
            ActionTrackParam {
                name: "action_0_track_1".to_string(),
                action_id: action_0.id,
                started_at: query_started_at_gte,
                duration: Some(180),
            },
            ActionTrackParam {
                name: "action_0_track_2".to_string(),
                action_id: action_0.id,
                started_at: query_started_at_lte,
                duration: Some(300),
            },
            ActionTrackParam {
                name: "_action_0_track_3".to_string(),
                action_id: action_0.id,
                started_at: query_started_at_lte + Duration::seconds(1),
                duration: Some(550),
            },
            ActionTrackParam {
                name: "action_1_track_0".to_string(),
                action_id: action_1.id,
                started_at: query_started_at_lte,
                duration: Some(350),
            },
        ],
        &user,
        &db,
    )
    .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/action_tracks/aggregation?started_at_gte={}&started_at_lte={}",
            query_started_at_gte.format("%Y-%m-%dT%H:%M:%SZ"),
            query_started_at_lte.format("%Y-%m-%dT%H:%M:%SZ")
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: ActionTrackAggregation = test::read_body_json(resp).await;

    let expected = ActionTrackAggregation {
        durations_by_action: vec![
            ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_tracks.get("action_0_track_1").unwrap().duration.unwrap()
                    + action_tracks.get("action_0_track_2").unwrap().duration.unwrap(),
                count: 2,
            },
            ActionTrackAggregationDuration {
                action_id: action_1.id,
                duration: action_tracks.get("action_1_track_0").unwrap().duration.unwrap(),
                count: 1,
            },
        ],
    };

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/aggregation")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
