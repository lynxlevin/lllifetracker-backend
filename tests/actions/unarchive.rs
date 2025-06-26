use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use use_cases::my_way::actions::types::ActionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};
use entities::action;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/unarchive", action.id))
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

    let restored_action = action::Entity::find_by_id(action.id)
        .one(&db)
        .await?
        .unwrap();
    assert_eq!(restored_action.user_id, user.id);
    assert_eq!(restored_action.archived, false);
    assert_eq!(ActionVisible::from(restored_action), res);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, .. } = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action = factory::action(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::put()
        .uri(&format!("/api/actions/{}/unarchive", action.id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
