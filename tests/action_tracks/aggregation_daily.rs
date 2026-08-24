use std::collections::HashMap;

use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Days};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::action_tracks::types::{ActionTrackAggregationDuration, ActionTrackDailyAggregationItem};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let actions = create_actions(
        vec![
            ActionParam { name: "action_0", ..Default::default() },
            ActionParam { name: "action_1", ..Default::default() },
            ActionParam { name: "_action_2", ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let action_0 = actions.get("action_0").unwrap();
    let action_1 = actions.get("action_1").unwrap();
    let target = DateTime::parse_from_rfc3339("2025-01-31T15:00:00Z").unwrap();
    let action_tracks = create_action_tracks(
        vec![
            ActionTrackParam {
                name: "_action_0_track_0",
                action_id: action_0.id,
                started_at: target.checked_sub_days(Days::new(1)).unwrap(),
                duration: Some(120),
            },
            ActionTrackParam {
                name: "action_0_track_1",
                action_id: action_0.id,
                started_at: target,
                duration: Some(180),
            },
            ActionTrackParam {
                name: "action_0_track_2",
                action_id: action_0.id,
                started_at: target.checked_add_days(Days::new(27)).unwrap(),
                duration: Some(300),
            },
            ActionTrackParam {
                name: "_action_0_track_3",
                action_id: action_0.id,
                started_at: target.checked_add_days(Days::new(28)).unwrap(),
                duration: Some(550),
            },
            ActionTrackParam {
                name: "action_1_track_0",
                action_id: action_1.id,
                started_at: target,
                duration: Some(350),
            },
        ],
        &user,
        &db,
    )
    .await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/aggregation/daily?year_month=202502")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: HashMap<String, Vec<ActionTrackDailyAggregationItem>> = test::read_body_json(resp).await;

    let expected_items = vec![
        ActionTrackDailyAggregationItem {
            date: 28,
            aggregation: vec![ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_tracks.get("action_0_track_2").unwrap().duration.unwrap(),
                count: 1,
            }],
        },
        ActionTrackDailyAggregationItem {
            date: 1,
            aggregation: vec![
                ActionTrackAggregationDuration {
                    action_id: action_0.id,
                    duration: action_tracks.get("action_0_track_1").unwrap().duration.unwrap(),
                    count: 1,
                },
                ActionTrackAggregationDuration {
                    action_id: action_1.id,
                    duration: action_tracks.get("action_1_track_0").unwrap().duration.unwrap(),
                    count: 1,
                },
            ],
        },
    ];
    let mut expected = HashMap::new();
    expected.insert("202502".to_string(), expected_items);

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/aggregation/daily?year_month=202506")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
