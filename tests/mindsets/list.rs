use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};

use super::super::utils::init_app;
use common::factory::{self, *};
use types::*;

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let mindset_0 = factory::mindset(user.id)
        .name("mindset_0".to_string())
        .insert(&db)
        .await?;
    let mindset_1 = factory::mindset(user.id)
        .name("mindset1".to_string())
        .description(Some("mindset1".to_string()))
        .insert(&db)
        .await?;
    let _archived_mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::get().uri("/api/mindsets").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<MindsetVisible> = test::read_body_json(resp).await;
    let expected = vec![
        MindsetVisible::from(mindset_0),
        MindsetVisible::from(mindset_1),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_show_archived_only() -> Result<(), DbErr> {
    let (app, db) = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _mindset = factory::mindset(user.id).insert(&db).await?;
    let archived_mindset = factory::mindset(user.id).archived(true).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri("/api/mindsets?show_archived_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<MindsetVisible> = test::read_body_json(resp).await;
    let expected = vec![MindsetVisible::from(archived_mindset)];

    assert_eq!(body.len(), expected.len());
    assert_eq!(body[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let (app, _) = init_app().await?;

    let req = test::TestRequest::get().uri("/api/mindsets").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
