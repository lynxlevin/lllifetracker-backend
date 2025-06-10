use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::{action, action_track};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::NO_CONTENT);

    let action_track_in_db = action_track::Entity::find_by_id(action_track.id)
        .one(&db)
        .await?;
    assert!(action_track_in_db.is_none());

    let action_in_db = action::Entity::find_by_id(action.id).one(&db).await?;
    assert!(action_in_db.is_some());

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;
    let action_track = factory::action_track(user.id)
        .action_id(action.id)
        .insert(&db)
        .await?;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/action_tracks/{}", action_track.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
