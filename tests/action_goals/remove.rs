use actix_web::{http, test, HttpMessage};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::action_goals::types::ActionGoalSetNewRequest;
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, ActionGoalFactory};
use entities::action_goal;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_goal = factory::action_goal(user.id, action.id)
        .from_date(
            DateTime::parse_from_rfc3339("2025-07-01T00:00:00Z")
                .unwrap()
                .date_naive(),
        )
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_goals?action_id={}", action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let action_goal_in_db = action_goal::Entity::find_by_id(action_goal.id)
        .one(&db)
        .await?
        .unwrap();
    let user_yesterday = (Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
        - Duration::days(1))
    .date_naive();
    assert_eq!(Some(user_yesterday), action_goal_in_db.to_date);

    Ok(())
}

#[actix_web::test]
async fn from_date_is_today() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    // MEMO: This test is flakey just around midnight, but the probability is so low I don't freeze now function.
    let action_goal = factory::action_goal(user.id, action.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_goals?action_id={}", action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let action_goal_in_db = action_goal::Entity::find_by_id(action_goal.id)
        .one(&db)
        .await?;
    assert!(action_goal_in_db.is_none());

    Ok(())
}

#[actix_web::test]
async fn no_active_goal() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_goals?action_id={}", action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalSetNewRequest {
            action_id: Uuid::now_v7(),
            duration_seconds: None,
            count: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn other_users_action() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let other_user = factory::user().insert(&db).await?;
    let other_action = factory::action(other_user.id).insert(&db).await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_goals?action_id={}", other_action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    Ok(())
}
