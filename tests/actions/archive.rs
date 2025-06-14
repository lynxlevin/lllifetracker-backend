use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use super::super::utils::init_app;
use common::factory;
use entities::action;
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
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
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/archive", action.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
