use std::collections::HashMap;

use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::action_tracks::types::{
    ActionTrackAggregationDuration, ActionTrackDailyAggregationItem,
};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action_0 = factory::action(user.id).insert(&db).await?;
    let action_1 = factory::action(user.id).insert(&db).await?;
    let _action_2 = factory::action(user.id).insert(&db).await?;
    let target = DateTime::parse_from_rfc3339("2025-02-01T15:00:00").unwrap();
    let _action_0_track_0 = factory::action_track(user.id)
        .started_at((target - Duration::days(1)).into())
        .duration(Some(120))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_0_track_1 = factory::action_track(user.id)
        .started_at(target.into())
        .duration(Some(180))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_0_track_2 = factory::action_track(user.id)
        .started_at((target + Duration::days(27)).into())
        .duration(Some(300))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let _action_0_track_3 = factory::action_track(user.id)
        .started_at((target + Duration::days(28)).into())
        .duration(Some(550))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_1_track_0 = factory::action_track(user.id)
        .started_at(target.into())
        .duration(Some(350))
        .action_id(action_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/action_tracks/aggregation/daily?year_month={}",
            // FIXME: These are naive datetimes, so maybe flakey.
            target.date_naive().format("%Y%m"),
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: HashMap<String, Vec<ActionTrackDailyAggregationItem>> =
        test::read_body_json(resp).await;

    let mut expected_items = vec![
        ActionTrackDailyAggregationItem {
            date: 1,
            aggregation: vec![
                ActionTrackAggregationDuration {
                    action_id: action_0.id,
                    duration: action_0_track_1.duration.unwrap(),
                    count: 1,
                },
                ActionTrackAggregationDuration {
                    action_id: action_1.id,
                    duration: action_1_track_0.duration.unwrap(),
                    count: 1,
                },
            ],
        },
        ActionTrackDailyAggregationItem {
            date: 2,
            aggregation: vec![ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_0_track_2.duration.unwrap(),
                count: 1,
            }],
        },
    ];
    expected_items.extend(
        (3..29)
            .map(|date| ActionTrackDailyAggregationItem {
                date,
                aggregation: vec![],
            })
            .collect::<Vec<_>>(),
    );
    assert_eq!(28, expected_items.len());
    let mut expected = HashMap::new();
    expected.insert("202502".to_string(), expected_items);

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/aggregation/daily")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
