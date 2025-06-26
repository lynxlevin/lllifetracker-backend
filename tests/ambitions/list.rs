use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::ambitions::types::AmbitionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let ambition_0 = factory::ambition(user.id)
        .name("ambition_0".to_string())
        .insert(&db)
        .await?;
    let ambition_1 = factory::ambition(user.id)
        .name("ambition1".to_string())
        .description(Some("ambition1".to_string()))
        .insert(&db)
        .await?;
    let _archived_ambition = factory::ambition(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get().uri("/api/ambitions").to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: Vec<AmbitionVisible> = test::read_body_json(resp).await;
    let expected = vec![
        AmbitionVisible::from(ambition_0),
        AmbitionVisible::from(ambition_1),
    ];

    assert_eq!(res.len(), expected.len());
    assert_eq!(res[0], expected[0]);
    assert_eq!(res[1], expected[1]);

    Ok(())
}

#[actix_web::test]
async fn happy_path_show_archived_only() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let _ambition = factory::ambition(user.id).insert(&db).await?;
    let archived_ambition = factory::ambition(user.id)
        .archived(true)
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri("/api/ambitions?show_archived_only=true")
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let body: Vec<AmbitionVisible> = test::read_body_json(resp).await;
    let expected = vec![AmbitionVisible::from(archived_ambition)];

    assert_eq!(body.len(), expected.len());
    assert_eq!(body[0], expected[0]);

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, ..} = init_app().await?;

    let req = test::TestRequest::get().uri("/api/ambitions").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
