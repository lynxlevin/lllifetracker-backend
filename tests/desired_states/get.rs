use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id)
        .description(Some("DesiredState".to_string()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/desired_states/{}", desired_state.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: DesiredStateVisible = test::read_body_json(resp).await;
    assert_eq!(res, DesiredStateVisible::from(desired_state));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let desired_state = factory::desired_state(user.id).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/desired_states/{}", desired_state.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
