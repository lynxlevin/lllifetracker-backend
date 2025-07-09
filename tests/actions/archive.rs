use actix_web::{http, test, HttpMessage};
use chrono::DateTime;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::actions::types::ActionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, ActionTrackFactory, UserFactory};
use entities::{action, user};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/archive", action.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action.id);
    assert_eq!(res.name, action.name.clone());
    assert_eq!(res.description, action.description.clone());
    assert_eq!(res.created_at, action.created_at);
    assert!(res.updated_at > action.updated_at);

    let action_in_db = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(action_in_db.archived, true);
    assert_eq!(ActionVisible::from(action_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_update_user_first_track_at() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user()
        .first_track_at(Some(
            DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap(),
        ))
        .insert(&db)
        .await?;
    let action_0 = factory::action(user.id).insert(&db).await?;
    let action_1 = factory::action(user.id).insert(&db).await?;
    let _action_track_0 = factory::action_track(user.id)
        .action_id(action_0.id)
        .started_at(DateTime::parse_from_rfc3339("2025-07-08T00:00:00Z").unwrap())
        .insert(&db)
        .await?;
    let action_track_1 = factory::action_track(user.id)
        .action_id(action_1.id)
        .started_at(DateTime::parse_from_rfc3339("2025-07-09T00:00:00Z").unwrap())
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/archive", action_0.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let user_in_db = user::Entity::find_by_id(user.id).one(&db).await?.unwrap();
    assert_eq!(user_in_db.first_track_at, Some(action_track_1.started_at));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/archive", action.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
