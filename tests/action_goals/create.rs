use actix_web::{http, test, HttpMessage};
use chrono::{FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::action_goals::types::{ActionGoalCreateRequest, ActionGoalVisible};
use uuid::Uuid;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, ActionFactory};
use entities::{action_goal, sea_orm_active_enums::ActionTrackType};

#[actix_web::test]
async fn happy_path_time_span() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let user_today = Utc::now()
        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
        .date_naive();
    let duration_seconds = Some(3600);

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalCreateRequest {
            action_id: action.id,
            duration_seconds,
            count: None,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionGoalVisible = test::read_body_json(res).await;
    assert_eq!(res.from_date, user_today);
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
async fn happy_path_count() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id)
        .track_type(ActionTrackType::Count)
        .insert(&db)
        .await?;

    let user_today = Utc::now()
        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
        .date_naive();
    let count = Some(5);

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalCreateRequest {
            action_id: action.id,
            duration_seconds: None,
            count,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::CREATED);

    let res: ActionGoalVisible = test::read_body_json(res).await;
    assert_eq!(res.from_date, user_today);
    assert_eq!(res.duration_seconds, None);
    assert_eq!(res.count, count);

    let action_goal_in_db = action_goal::Entity::find_by_id(res.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_goal_in_db.user_id, user.id);
    assert_eq!(ActionGoalVisible::from(action_goal_in_db), res);

    Ok(())
}

// #[actix_web::test]
// async fn invalidate_existing_action_goal() -> Result<(), DbErr> {
//     let Connections { app, db, .. } = init_app().await?;
//     let user = factory::user().insert(&db).await?;
//     let action = factory::action(user.id).insert(&db).await?;
//     let existing_goal = factory::action_goal(user.id, action.id).await?;
//     let other_action = factory::action(user.id).insert(&db).await?;
//     let existing_other_action_goal = factory::action_goal(user.id, other_action.id).await?;

//     let req = test::TestRequest::post()
//         .uri("/api/action_goals")
//         .set_json(ActionGoalCreateRequest {
//             action_id: action.id,
//             from_date: Utc::now().date_naive(),
//             duration_seconds: Some(3600),
//             count: None,
//         })
//         .to_request();
//     req.extensions_mut().insert(user.clone());

//     let res = test::call_service(&app, req).await;
//     assert_eq!(res.status(), http::StatusCode::CREATED);

//     let existing_goal_in_db = action_goal::Entity::find_by_id(existing_goal.id)
//         .one(&db)
//         .await?
//         .unwrap();
//     assert_eq!();

//     Ok(())
// }

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/action_goals")
        .set_json(ActionGoalCreateRequest {
            action_id: Uuid::now_v7(),
            duration_seconds: None,
            count: None,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod not_found {
    use super::*;

    #[actix_web::test]
    async fn other_users_action() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let other_user = factory::user().insert(&db).await?;
        let other_action = factory::action(other_user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri("/api/action_goals")
            .set_json(ActionGoalCreateRequest {
                action_id: other_action.id,
                duration_seconds: None,
                count: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);

        Ok(())
    }
}

mod bad_request {
    use super::*;

    #[actix_web::test]
    async fn duration_seconds_none_for_time_span_action() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri("/api/action_goals")
            .set_json(ActionGoalCreateRequest {
                action_id: action.id,
                duration_seconds: None,
                count: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn count_not_none_for_time_span_action() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        let req = test::TestRequest::post()
            .uri("/api/action_goals")
            .set_json(ActionGoalCreateRequest {
                action_id: action.id,
                duration_seconds: None,
                count: Some(5),
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn count_none_for_count_action() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id)
            .track_type(ActionTrackType::Count)
            .insert(&db)
            .await?;

        let req = test::TestRequest::post()
            .uri("/api/action_goals")
            .set_json(ActionGoalCreateRequest {
                action_id: action.id,
                duration_seconds: None,
                count: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[actix_web::test]
    async fn duration_seconds_not_none_for_count_action() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id)
            .track_type(ActionTrackType::Count)
            .insert(&db)
            .await?;

        let req = test::TestRequest::post()
            .uri("/api/action_goals")
            .set_json(ActionGoalCreateRequest {
                action_id: action.id,
                duration_seconds: Some(3600),
                count: None,
            })
            .to_request();
        req.extensions_mut().insert(user.clone());

        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
