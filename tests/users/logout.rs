use actix_web::{http, test};
use sea_orm::DbErr;

use crate::utils::{init_app, Connections};

#[actix_web::test]
#[ignore]
async fn happy_path() -> Result<(), DbErr> {
    unimplemented!("This is checked in integration.rs.");
}

#[actix_web::test]
async fn ok_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, .. } = init_app().await?;

    let req = test::TestRequest::post()
        .uri("/api/users/logout")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    Ok(())
}
