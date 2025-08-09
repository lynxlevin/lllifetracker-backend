use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::action_tracks::types::{
    ActionTrackAggregation, ActionTrackAggregationDuration,
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
    let jst_now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
    let action_0_track_0 = factory::action_track(user.id)
        .started_at(jst_now - Duration::days(1))
        .duration(Some(120))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let _action_0_track_1 = factory::action_track(user.id)
        .started_at(jst_now)
        .duration(Some(180))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_0_track_2 = factory::action_track(user.id)
        .started_at(jst_now + Duration::days(1))
        .duration(Some(300))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let _action_0_track_3 = factory::action_track(user.id)
        .started_at(jst_now + Duration::days(2))
        .duration(Some(550))
        .action_id(action_0.id)
        .insert(&db)
        .await?;
    let action_1_track_0 = factory::action_track(user.id)
        .started_at(jst_now + Duration::days(1))
        .duration(Some(350))
        .action_id(action_1.id)
        .insert(&db)
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
                duration: action_0_track_0.duration.unwrap() + action_0_track_2.duration.unwrap(),
                count: 2,
            },
            ActionTrackAggregationDuration {
                action_id: action_1.id,
                duration: action_1_track_0.duration.unwrap(),
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
    let action_0 = factory::action(user.id).insert(&db).await?;
    let action_1 = factory::action(user.id).insert(&db).await?;
    let _action_2 = factory::action(user.id).insert(&db).await?;
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
                duration: action_0_track_1.duration.unwrap() + action_0_track_2.duration.unwrap(),
                count: 2,
            },
            ActionTrackAggregationDuration {
                action_id: action_1.id,
                duration: action_1_track_0.duration.unwrap(),
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
