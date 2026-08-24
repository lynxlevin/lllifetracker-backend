use actix_web::{http, test, HttpMessage};
use chrono::NaiveDate;
use entities::action::ActionTrackType;
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::actions::types::ActionVisibleWithGoal;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db.db).await?;
    let mut actions = create_actions(
        vec![
            ActionParam { name: "action_0", track_type: None, archived: false, ..Default::default() },
            ActionParam {
                name: "action_1",
                track_type: Some(ActionTrackType::Count),
                archived: false,
                ..Default::default()
            },
            ActionParam { name: "archived_action", track_type: None, archived: true, ..Default::default() },
        ],
        &user,
        &db,
    )
    .await?;
    let action_0 = actions.remove("action_0").unwrap();
    let action_1 = actions.remove("action_1").unwrap();
    let archived_action = actions.remove("archived_action").unwrap();
    let action_goal_0 = factory::action_goal(user.id, action_0.id)
        .duration_seconds(Some(3600))
        .insert(&db.db)
        .await?;
    let action_goal_1 = factory::action_goal(user.id, action_1.id)
        .count(Some(5))
        .insert(&db.db)
        .await?;

    let req = test::TestRequest::get().uri("/api/actions").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionVisibleWithGoal> = test::read_body_json(resp).await;

    let expected = vec![
        ActionVisibleWithGoal::from((action_0, Some(action_goal_0))),
        ActionVisibleWithGoal::from((action_1, Some(action_goal_1))),
        ActionVisibleWithGoal::from((archived_action, None)),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::get().uri("/api/actions").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}

mod list_with_goal {
    use super::*;

    #[actix_web::test]
    async fn return_only_active_goal() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db.db).await?;
        let action = factory::action(user.id).insert(&db.db).await?;
        let _inactive_action_goal = factory::action_goal(user.id, action.id)
            .from_date(NaiveDate::from_ymd_opt(2025, 8, 12).unwrap())
            .to_date(Some(NaiveDate::from_ymd_opt(2025, 8, 12).unwrap()))
            .insert(&db.db)
            .await?;
        let active_action_goal = factory::action_goal(user.id, action.id)
            .from_date(NaiveDate::from_ymd_opt(2025, 8, 13).unwrap())
            .insert(&db.db)
            .await?;

        let req = test::TestRequest::get().uri("/api/actions").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let res: Vec<ActionVisibleWithGoal> = test::read_body_json(resp).await;

        let expected = vec![ActionVisibleWithGoal::from((action, Some(active_action_goal)))];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }

    #[actix_web::test]
    async fn return_action_with_no_active_goal() -> Result<(), DbErr> {
        let Connections { app, db, .. } = init_app().await?;
        let user = factory::user().insert(&db.db).await?;
        let action_with_no_goal = factory::action(user.id)
            .name("no_goal".to_string())
            .insert(&db.db)
            .await?;
        let action_with_only_inactive_goal = factory::action(user.id).insert(&db.db).await?;
        let _inactive_action_goal = factory::action_goal(user.id, action_with_only_inactive_goal.id)
            .from_date(NaiveDate::from_ymd_opt(2025, 8, 12).unwrap())
            .to_date(Some(NaiveDate::from_ymd_opt(2025, 8, 12).unwrap()))
            .insert(&db.db)
            .await?;

        let req = test::TestRequest::get().uri("/api/actions").to_request();
        req.extensions_mut().insert(user.clone());

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let res: Vec<ActionVisibleWithGoal> = test::read_body_json(resp).await;

        let expected = vec![
            ActionVisibleWithGoal::from((action_with_no_goal, None)),
            ActionVisibleWithGoal::from((action_with_only_inactive_goal, None)),
        ];
        dbg!(&res);

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }
}
