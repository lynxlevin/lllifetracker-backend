use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration, FixedOffset};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let query_started_at_gte: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2025-01-27T00:00:00Z").unwrap();
    let query_started_at_lte: DateTime<FixedOffset> =
        DateTime::parse_from_rfc3339("2025-01-27T23:59:59Z").unwrap();
    let action_0 = factory::action(user.id).insert(&db).await?;
    let action_1 = factory::action(user.id).insert(&db).await?;
    let _action_0_track_0 = factory::action_track(user.id)
        .started_at(query_started_at_gte - Duration::seconds(1))
        .duration(Some(120))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_0_track_1 = factory::action_track(user.id)
        .started_at(query_started_at_gte)
        .duration(Some(180))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_0_track_2 = factory::action_track(user.id)
        .started_at(query_started_at_lte)
        .duration(Some(300))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let _action_0_track_3 = factory::action_track(user.id)
        .started_at(query_started_at_lte + Duration::seconds(1))
        .duration(Some(550))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_1_track_0 = factory::action_track(user.id)
        .started_at(query_started_at_lte)
        .duration(Some(350))
        .action_id(action_1.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&*format!(
            "/api/action_tracks/aggregation?started_at_gte={}&started_at_lte={}",
            query_started_at_gte.format("%Y-%m-%dT%H:%M:%SZ"),
            query_started_at_lte.format("%Y-%m-%dT%H:%M:%SZ")
        ))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    dbg!(&resp.response().body());
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: ActionTrackAggregation = test::read_body_json(resp).await;

    let expected = ActionTrackAggregation {
        durations_by_action: vec![
            ActionTrackAggregationDuration {
                action_id: action_0.id,
                duration: action_0_track_1.duration.unwrap() + action_0_track_2.duration.unwrap(),
            },
            ActionTrackAggregationDuration {
                action_id: action_1.id,
                duration: action_1_track_0.duration.unwrap(),
            },
        ],
    };

    assert_eq!(res, expected);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get()
        .uri("/api/action_tracks/aggregation")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
