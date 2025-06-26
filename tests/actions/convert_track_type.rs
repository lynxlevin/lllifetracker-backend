use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::actions::types::{ActionTrackTypeConversionRequest, ActionVisible};

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{action, sea_orm_active_enums::ActionTrackType};

#[actix_web::test]
async fn happy_path_time_span_to_count() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id)
        .track_type(ActionTrackType::TimeSpan)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/track_type", action.id))
        .set_json(ActionTrackTypeConversionRequest {
            track_type: ActionTrackType::Count,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action.id);
    assert_eq!(res.name, action.name);
    assert_eq!(res.description, action.description);
    assert_eq!(res.trackable, action.trackable);
    assert_eq!(res.color, action.color);
    assert_eq!(res.track_type, ActionTrackType::Count);
    assert_eq!(res.created_at, action.created_at);
    assert!(res.updated_at > action.updated_at);

    let action_in_db = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(action_in_db.archived, action.archived);
    assert_eq!(ActionVisible::from(action_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_count_to_time_span() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id)
        .track_type(ActionTrackType::Count)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/track_type", action.id))
        .set_json(ActionTrackTypeConversionRequest {
            track_type: ActionTrackType::TimeSpan,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action.id);
    assert_eq!(res.name, action.name);
    assert_eq!(res.description, action.description);
    assert_eq!(res.trackable, action.trackable);
    assert_eq!(res.color, action.color);
    assert_eq!(res.track_type, ActionTrackType::TimeSpan);
    assert_eq!(res.created_at, action.created_at);
    assert!(res.updated_at > action.updated_at);

    let action_in_db = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(action_in_db.archived, action.archived);
    assert_eq!(ActionVisible::from(action_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn happy_path_no_change() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id)
        .track_type(ActionTrackType::TimeSpan)
        .insert(&db)
        .await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/track_type", action.id))
        .set_json(ActionTrackTypeConversionRequest {
            track_type: ActionTrackType::TimeSpan,
        })
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let res: ActionVisible = test::read_body_json(res).await;
    assert_eq!(res.id, action.id);
    assert_eq!(res.name, action.name);
    assert_eq!(res.description, action.description);
    assert_eq!(res.trackable, action.trackable);
    assert_eq!(res.color, action.color);
    assert_eq!(res.track_type, ActionTrackType::TimeSpan);
    assert_eq!(res.created_at, action.created_at);
    assert_eq!(res.updated_at, action.updated_at);

    let action_in_db = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(action_in_db.user_id, user.id);
    assert_eq!(action_in_db.archived, action.archived);
    assert_eq!(ActionVisible::from(action_in_db), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/track_type", action.id))
        .set_json(ActionTrackTypeConversionRequest {
            track_type: ActionTrackType::Count,
        })
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
