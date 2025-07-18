use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::actions::types::ActionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let action_0 = factory::action(user.id)
        .name("action_0".to_string())
        .insert(&db)
        .await?;
    let action_1 = factory::action(user.id)
        .name("action_1".to_string())
        .insert(&db)
        .await?;
    let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::get().uri("/api/actions").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionVisible> = test::read_body_json(resp).await;

    let expected = vec![ActionVisible::from(action_0), ActionVisible::from(action_1)];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_show_archived_only() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _action = factory::action(user.id).insert(&db).await?;
    let archived_action = factory::action(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri("/api/actions?show_archived_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<ActionVisible> = test::read_body_json(resp).await;

    let expected = vec![ActionVisible::from(archived_action)];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, ..} = init_app().await?;

    let req = test::TestRequest::get().uri("/api/actions").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
