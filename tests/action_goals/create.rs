use actix_web::{http, test, HttpMessage};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::action_goals::types::{ActionGoalCreateRequest, ActionGoalVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory;
use entities::action_goal;

#[actix_web::test]
async fn happy_path_time_span() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let today = Utc::now().date_naive();
    let duration_seconds = Some(3600);

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalCreateRequest {
            action_id: action.id,
            from_date: today,
            duration_seconds,
            count: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionGoalVisible = test::read_body_json(res).await;
    assert_eq!(res.from_date, today);
    assert_eq!(res.duration_seconds, duration_seconds);
    assert_eq!(res.count, None);

    let action_goal_in_db = action_goal::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_goal_in_db.user_id, user.id);
    assert_eq!(ActionGoalVisible::from(action_goal_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalCreateRequest {
            action_id: Uuid::now_v7(),
            from_date: Utc::now().date_naive(),
            duration_seconds: None,
            count: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
