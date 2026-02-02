use actix_web::{http, test, HttpMessage};
use sea_orm::{ActiveModelTrait, DbErr};
use use_cases::my_way::directions::types::DirectionVisible;

use crate::utils::Connections;

use super::super::utils::init_app;
use common::factory::{self, *};

#[actix_web::test]
async fn happy_path() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id)
        .description(Some("Direction".to_string()))
        .insert(&db)
        .await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/directions/{}", direction.id))
        .to_request();
    req.extensions_mut().insert(user.clone());

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);

    let res: DirectionVisible = test::read_body_json(resp).await;
    assert_eq!(res, DirectionVisible::from(direction));

    Ok(())
}

#[actix_web::test]
async fn unauthorized_if_not_logged_in() -> Result<(), DbErr> {
    let Connections { app, db, ..} = init_app().await?;
    let user = factory::user().insert(&db).await?;
    let direction = factory::direction(user.id).insert(&db).await?;

    let req = test::TestRequest::get()
        .uri(&format!("/api/directions/{}", direction.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    Ok(())
}
