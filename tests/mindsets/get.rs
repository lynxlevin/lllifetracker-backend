use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id)
        .description(Some("mindset".to_string()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: MindsetVisible = test::read_body_json(resp).await;
    assert_eq!(res, MindsetVisible::from(mindset));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset = factory::mindset(user.id).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/mindsets/{}", mindset.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
